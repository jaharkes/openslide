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

//! Cache that evicts objects based on total object size.

use crate::lrucache::LruCache;
use std::ffi::c_void;
use std::hash::Hash;
use std::os::raw::c_int;
use std::ptr;
use std::sync::Arc;

#[derive(Hash, Eq, PartialEq)]
pub struct CacheKey(*const c_void, i64, i64);

// Special CacheEntry struct so we can free the cached data
// that was allocated in the C code with g_slice_alloc.
// Cleaner solutions would be,
// - add a dealloc callback
// - expose a Rust allocator to C.
//
// However, either way would require changes to existing code.
pub struct CacheEntry {
    data: *mut c_void,
    size: usize,
}

impl Drop for CacheEntry {
    fn drop(&mut self) {
        #[link(name = "glib-2.0")]
        extern "C" {
            fn g_slice_free1(size: usize, data: *mut c_void);
        }
        unsafe {
            g_slice_free1(self.size, self.data);
        }
    }
}

//
// Functions called from C
//

pub const _OPENSLIDE_USEFUL_CACHE_SIZE: usize = 1024 * 1024 * 32;

// constructor/destructor
#[no_mangle]
pub extern "C" fn _openslide_cache_create(
    capacity_in_bytes: c_int,
) -> *mut LruCache<CacheKey, CacheEntry> {
    Box::into_raw(Box::new(LruCache::new(capacity_in_bytes as usize)))
}

#[no_mangle]
pub extern "C" fn _openslide_cache_destroy(cache: *mut LruCache<CacheKey, CacheEntry>) {
    if !cache.is_null() {
        unsafe {
            Box::from_raw(cache);
        };
    }
}

#[no_mangle]
pub extern "C" fn _openslide_cache_get_capacity(
    cache: *const LruCache<CacheKey, CacheEntry>,
) -> c_int {
    let cache = unsafe {
        assert!(!cache.is_null());
        &*cache
    };
    cache.get_capacity() as c_int
}

#[no_mangle]
pub extern "C" fn _openslide_cache_set_capacity(
    cache: *const LruCache<CacheKey, CacheEntry>,
    capacity_in_bytes: c_int,
) {
    let cache = unsafe {
        assert!(!cache.is_null());
        &*cache
    };
    cache.set_capacity(capacity_in_bytes as usize);
}

#[no_mangle]
pub extern "C" fn _openslide_cache_put(
    cache: *const LruCache<CacheKey, CacheEntry>,
    plane: *const c_void,
    x: i64,
    y: i64,
    data: *mut c_void,
    size_in_bytes: c_int,
    entry: *mut *const CacheEntry,
) {
    let cache = unsafe {
        assert!(!cache.is_null());
        &*cache
    };
    let size = size_in_bytes as usize;
    let key = CacheKey(plane, x, y);
    let val = CacheEntry { data, size };

    // put a copy in the cache, get back a referenced copy
    let arc = cache.put(key, val, size);

    // and return a reference to the caller
    if !entry.is_null() {
        unsafe {
            ptr::write(entry, Arc::into_raw(arc));
        }
    }
}

#[no_mangle]
pub extern "C" fn _openslide_cache_get(
    cache: *const LruCache<CacheKey, CacheEntry>,
    plane: *const c_void,
    x: i64,
    y: i64,
    entry: *mut *const CacheEntry,
) -> *const c_void {
    let cache = unsafe {
        assert!(!cache.is_null());
        &*cache
    };
    let key = CacheKey(plane, x, y);

    match cache.get(&key) {
        Some(val) => unsafe {
            assert!(!entry.is_null());
            ptr::write(entry, Arc::into_raw(val));
            (*(*entry)).data
        },
        None => ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn _openslide_cache_entry_unref(entry: *mut CacheEntry) {
    if !entry.is_null() {
        unsafe {
            Arc::from_raw(entry);
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache() {
        let mut entry: *const CacheEntry = std::ptr::null();
        let entry_ptr: *mut *const CacheEntry = &mut entry;
        let null_ptr: *mut *const CacheEntry = std::ptr::null_mut();
        let null = std::ptr::null_mut();

        let cache = _openslide_cache_create(200 * 1024 * 1024);

        // check key that was never inserted in cache
        assert_eq!(_openslide_cache_get(cache, null, 0, 0, entry_ptr), null);
        unsafe {
            assert!((*entry_ptr).is_null());
        }

        // insert 100,000 100MB chunks into a 200MB cache.
        for i in 1..100_000 {
            unsafe {
                extern "C" {
                    fn g_slice_alloc(size: usize) -> *mut c_void;
                }
                let size = 100 * 1024 * 1024;
                let data = g_slice_alloc(size);
                _openslide_cache_put(cache, null, i, 0, data, size as c_int, null_ptr);
            }
        }

        _openslide_cache_destroy(cache);
    }
}
