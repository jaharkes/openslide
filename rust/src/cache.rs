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

//! Wrapper around [`LruCache`] to provide FFI-bindings and deallocate
//! cached C objects that were allocated with `g_slice_alloc()`.
//!
//! [`LruCache`]: ../lrucache/index.html

use crate::lrucache::LruCache;
use std::ffi::c_void;
use std::hash::Hash;
use std::os::raw::c_int;
use std::ptr;
use std::sync::Arc;

// CacheKey is used to collect the parts of the key and make the function
// signatures more readable. But it isn't really exposed through the FFI
// API so there isn't much to document.
#[doc(hidden)]
#[derive(Hash, Eq, PartialEq)]
pub struct CacheKey(*const c_void, i64, i64);

/// A CacheEntry struct that wraps the C objects with a custom drop.
///
/// We use this so we free the cached data that was allocated with
/// g_slice_alloc in the calling C code.
///
/// Cleaner solutions would be,
/// - add a dealloc callback so we can have both allocation and
///   deallocations in the C code.
/// - expose a Rust object allocator to C, so allocations and
///   deallocations happen in the Rust part of the code.
///
/// However, either way would require changes to existing code.
#[derive(Hash, Eq, PartialEq)]
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

/// Useful cache size to allocate per open slide handle.
/// currently defaults to 32MB.
pub const _OPENSLIDE_USEFUL_CACHE_SIZE: usize = 1024 * 1024 * 32;

/// Create a new cache.
#[no_mangle]
pub extern "C" fn _openslide_cache_create(
    capacity_in_bytes: c_int,
) -> *mut LruCache<CacheKey, CacheEntry> {
    Box::into_raw(Box::new(LruCache::new(capacity_in_bytes as usize)))
}

/// Destroy a cache and drop all cached objects.
#[no_mangle]
pub extern "C" fn _openslide_cache_destroy(cache: *mut LruCache<CacheKey, CacheEntry>) {
    if !cache.is_null() {
        unsafe {
            Box::from_raw(cache);
        };
    }
}

/// Get the currently configured maximum cache size.
///
/// Wrapper around [`LruCache::get_capacity()`].
///
/// [`LruCache::get_capacity()`]: ../lrucache/struct.LruCache.html#method.get_capacity
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

/// Set the maximum cache size.
///
/// Wrapper around [`LruCache::set_capacity()`]
///
/// [`LruCache::set_capacity()`]: ../lrucache/struct.LruCache.html#method.set_capacity
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

/// Add an object to the cache.
///
/// Adds an object `data` that is `size_in_bytes` long to the cache in the
/// position indexed by (`plane`, `x`, `y`). This will evict anything that
/// is already stored in that location as well as the least recently accessed
/// items that exceed the configured cache size.
///
/// This function returns a reference to the cached `entry`, which must be
/// released with [`_openslide_cache_entry_unref()`].
///
/// [`_openslide_cache_entry_unref()`]: ./fn._openslide_cache_entry_unref.html
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

/// Find a cached object in the cache.
///
/// This function returns both pointer to the cached object data as well as
/// a reference to the cached `entry`, which must be released with
/// [`_openslide_cache_entry_unref()`].
///
/// [`_openslide_cache_entry_unref()`]: ./fn._openslide_cache_entry_unref.html
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
        None => unsafe {
            // should we even bother to null the entry?
            // it is the only reason this part of the code is marked 'unsafe'.
            assert!(!entry.is_null());
            ptr::write(entry, ptr::null());
            ptr::null()
        }
    }
}

/// Release a reference to a cached entry.
///
/// This allows the pointer to the associated data to be safely deallocated
/// when the object is dropped from the cache.
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
