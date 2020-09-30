//
//  OpenSlide, a library for reading whole slide image files
//
//  Copyright (c) 2020 Carnegie Mellon University
//  All rights reserved.
//
//  OpenSlide is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Lesser General Public License as
//  published by the Free Software Foundation, version 2.1.
//
//  OpenSlide is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//  GNU Lesser General Public License for more details.
//
//  You should have received a copy of the GNU Lesser General Public
//  License along with OpenSlide. If not, see
//  <http://www.gnu.org/licenses/>.
//
// SPDX-Licence-Identifier: LGPL-2.1-only
//

//! This is a wrapper around sha256 which helps compute format specific
//! 'quickhash' identifiers for whole slide files.
//!
//! # Examples
//!
//! ```
//! let hash: QuickHash1 = QuickHash1::new();
//!
//! // hash 'Openslide, a library ...'
//! hash.update_from_file_slice("src/hash.rs", 7, 56);
//!
//! println!("{}\n", hash.get_string().unwrap());
//!
//! assert_eq!(
//!     hash.get_string().unwrap(),
//!     "5744c6161af92b6a43441703179827e4b1788f10c23fa8c91e517827e29b8cd3"
//! );
//! ```

extern crate sha2;
use crate::util::FileSlice;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;

// size of the hexadecimal representation of a SHA256 hash including final '\0'
pub const _QUICKHASH1_CHECKSUMSIZE: usize = 32 * 2 + 1;

/// QuickHash1 helper implementation.
pub struct QuickHash1 {
    hasher: Option<Sha256>,
    // the checksum field allows an ffi returned checksum value to survive
    // for as long as `struct QuickHash1` lives.
    checksum: [u8; _QUICKHASH1_CHECKSUMSIZE],
}

impl QuickHash1 {
    /// Initialize a new QuickHash1 structure.
    fn new() -> QuickHash1 {
        QuickHash1 {
            hasher: Some(Sha256::new()),
            checksum: [0; _QUICKHASH1_CHECKSUMSIZE],
        }
    }

    /// Update state of the QuickHash1 object with the given data.
    fn update(&mut self, data: &[u8]) {
        if let Some(hasher) = self.hasher.as_mut() {
            hasher.update(data);
        }
    }

    /// Update state of the QuickHash1 object based on the content of
    /// the file between offset and offset + length.
    fn update_from_file_slice(
        &mut self,
        filename: &str,
        offset: u64,
        length: i64,
    ) -> io::Result<()> {
        if let Some(hasher) = self.hasher.as_mut() {
            let file = File::open(&filename)?;
            let mut fileslice = FileSlice::new(file, offset, length)?;
            std::io::copy(&mut fileslice, hasher)?;
        }
        Ok(())
    }

    /// Get a hexadecimal representation of the current QuickHash1 state.
    fn get_string(&mut self) -> Option<String> {
        let result = self.hasher.as_ref()?.clone().finalize();
        Some(format!("{:x}", result))
    }
}

#[cfg(test)]
mod tests {
    use super::QuickHash1;

    #[test]
    fn test_quickhash1_empty() {
        let mut hash = QuickHash1::new();

        assert_eq!(
            hash.get_string().unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_quickhash1_update() {
        let mut hash = QuickHash1::new();
        hash.update(b"Openslide hash test");

        assert_eq!(
            hash.get_string().unwrap(),
            "8e24777feb8cdab7bc3fa0eb2bac24c9bcc5cf87ba9e2b0ced08dd41c0533665"
        );
    }

    #[test]
    fn test_quickhash1_file_slice() {
        let mut hash = QuickHash1::new();

        // hash("OpenSlide, a library for reading whole slide image files")
        hash.update_from_file_slice("src/hash.rs", 7, 56).unwrap();

        assert_eq!(
            hash.get_string().unwrap(),
            "5744c6161af92b6a43441703179827e4b1788f10c23fa8c91e517827e29b8cd3"
        );
    }
}

/// Wrappers around `QuickHash1` to provide FFI-bindings for libopenslide.
pub mod ffi {
    extern crate glib_sys;
    use super::QuickHash1;
    use glib_sys::{g_file_error_from_errno, GError, GFileError, G_FILE_ERROR_INVAL};
    use std::ffi::{c_void, CStr, CString};
    use std::io::Write;
    use std::os::raw::{c_char, c_int};
    use std::ptr;
    use std::slice;

    // helper function to initialize glib GError object on failure
    fn set_gerror(err: *mut *mut GError, code: GFileError, msg: &str) -> bool {
        unsafe {
            let domain = glib_sys::g_file_error_quark();
            let msg = CString::new(msg).unwrap().into_raw();
            glib_sys::g_set_error_literal(err, domain, code as c_int, msg);
        }
        false
    }

    /// Create a new quickhash1 object
    #[no_mangle]
    pub extern "C" fn _openslide_hash_quickhash1_create() -> *mut QuickHash1 {
        Box::into_raw(Box::new(QuickHash1::new()))
    }

    /// Destroy quickhash1 object
    #[no_mangle]
    pub extern "C" fn _openslide_hash_destroy(hash: *mut QuickHash1) {
        if !hash.is_null() {
            unsafe {
                drop(Box::from_raw(hash));
            };
        }
    }

    /// Update hash to include datalen bytes from data.
    #[no_mangle]
    pub extern "C" fn _openslide_hash_data(
        hash: *mut QuickHash1,
        data: *const c_void,
        datalen: i32,
    ) {
        if !hash.is_null() && !data.is_null() && datalen > 0 {
            let hash = unsafe { &mut *hash };
            let data = unsafe { slice::from_raw_parts(data as *const u8, datalen as usize) };
            hash.update(data);
        }
    }

    /// Add string, including final '\0', to hash.
    #[no_mangle]
    pub extern "C" fn _openslide_hash_string(hash: *mut QuickHash1, string: *const c_char) {
        if !hash.is_null() {
            let hash = unsafe { &mut *hash };
            let data = unsafe {
                if !string.is_null() {
                    CStr::from_ptr(string).to_bytes_with_nul()
                } else {
                    &[0] // hash("")
                }
            };
            hash.update(data);
        }
    }

    /// Add content of file to hash.
    #[no_mangle]
    pub extern "C" fn _openslide_hash_file(
        hash: *mut QuickHash1,
        filename: *const c_char,
        err: *mut *mut glib_sys::GError,
    ) -> bool {
        _openslide_hash_file_part(hash, filename, 0, -1, err)
    }

    /// Inlucde of part of the file in hash.
    #[no_mangle]
    pub extern "C" fn _openslide_hash_file_part(
        hash: *mut QuickHash1,
        filename: *const c_char,
        offset: i64,
        size: i64,
        err: *mut *mut GError,
    ) -> bool {
        if hash.is_null() {
            return set_gerror(
                err,
                G_FILE_ERROR_INVAL,
                &String::from("Invalid argument, NULL hash"),
            );
        }
        if offset < 0 {
            return set_gerror(
                err,
                G_FILE_ERROR_INVAL,
                &String::from("Invalid argument, negative offset"),
            );
        }

        let hash = unsafe { &mut *hash };
        let filename = unsafe { CStr::from_ptr(filename).to_string_lossy().into_owned() };
        match hash.update_from_file_slice(&filename, offset as u64, size) {
            Ok(_) => true,
            Err(e) => {
                let code = unsafe {
                    let errno = e.raw_os_error().unwrap();
                    g_file_error_from_errno(errno)
                };
                set_gerror(err, code, &format!("{}", e))
            }
        }
    }

    /// Invalidate this hash. Use if this slide is unhashable for some reason.
    #[no_mangle]
    pub extern "C" fn _openslide_hash_disable(hash: *mut QuickHash1) {
        if !hash.is_null() {
            let hash = unsafe { &mut *hash };
            hash.hasher.take();
        }
    }

    /// Get hexadecimal representation of quickhash1 state.
    #[no_mangle]
    pub extern "C" fn _openslide_hash_get_string(hash: *mut QuickHash1) -> *const c_char {
        if hash.is_null() {
            return ptr::null();
        }
        let hash = unsafe { &mut *hash };
        match hash.get_string() {
            Some(result) => {
                write!(&mut hash.checksum[..], "{}", result).unwrap();
                hash.checksum.as_ptr() as *const c_char
            },
            None => ptr::null(),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_hash() {
            let hash = _openslide_hash_quickhash1_create();
            let data = CString::new("foobar").unwrap();

            _openslide_hash_data(hash, data.as_ptr() as *const c_void, 6);
            _openslide_hash_string(hash, data.as_ptr() as *const c_char);

            unsafe {
                assert_eq!(
                    CStr::from_ptr(_openslide_hash_get_string(hash)).to_bytes(),
                    &b"e44dbd702687312612b8e03ed5bee008f15b8abe62ebac51cafe73e20ab7c5ff"[..]
                );
            }

            _openslide_hash_disable(hash);
            assert_eq!(_openslide_hash_get_string(hash), ptr::null());

            _openslide_hash_destroy(hash);
        }
    }
}
