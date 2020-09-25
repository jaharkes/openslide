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

//! This is an implementation of a LRU Cache that evicts objects
//! based on the total size of the cached objects.
//!
//! # Examples
//!
//! ```
//! let cache: LruCache<u32, u32> = LruCache::new(200);
//!
//! cache.put(0, 0, 100);
//! cache.put(1, 1, 100);
//!
//! // Accessing the first entry brings it to the top the LRU
//! cache.get(&0);
//!
//! // this will push the least-recently-used entry out of the cache
//! cache.put(2, 2, 100);
//!
//! // second entry should be evicted from cache
//! assert!(cache.get(&1).is_none());
//!
//! // first and third entries should still exist in the cache
//! assert_eq!(cache.get(&0), Some(Arc::new(0)));
//! assert_eq!(cache.get(&2), Some(Arc::new(2)));
//! ```

extern crate linked_hash_map;
use linked_hash_map::LinkedHashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

// Used to hold references to cached entries and their size/weight
struct CacheItem<V> {
    entry: Arc<V>,
    size: usize,
}

// Cache stuff that can only be accessed while a `std::sync::Mutex` is held.
struct _LruCache<K, V> {
    lru: LinkedHashMap<K, CacheItem<V>>,
    capacity: usize,
    total_size: usize,
}

impl<K, V> _LruCache<K, V>
where
    K: Hash + Eq,
{
    // Drop entries to clear enough cache space to add `reserve` bytes.
    fn _shrink_to_fit(&mut self, reserve: usize) {
        while self.total_size + reserve > self.capacity {
            match self.lru.pop_front() {
                Some(val) => {
                    self.total_size -= val.1.size;
                }
                None => break,
            }
        }
    }
}

/// LRU cache implementation.
pub struct LruCache<K, V>(Mutex<_LruCache<K, V>>);

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq,
{
    /// Initialize a new LruCache, with the specified maximum size.
    pub fn new(capacity_in_bytes: usize) -> LruCache<K, V> {
        LruCache(Mutex::new(_LruCache {
            lru: LinkedHashMap::new(),
            capacity: capacity_in_bytes,
            total_size: 0,
        }))
    }

    /// Get configured LruCache maximum size
    ///
    /// **Note to self:** Maybe it would be more useful to return
    /// the total size of currently cached objects?
    pub fn get_capacity(&self) -> usize {
        let cache = self.0.lock().unwrap();

        cache.capacity
    }

    /// Set new LruCache maximum capacity
    ///
    /// Will discard least recently used objects that exceed the new
    /// size, can as such be used to empty the current cache.
    ///
    /// ```
    /// let saved = cache.get_capacity();
    /// cache.set_capacity(0);
    /// cache.set_capacity(saved);
    /// ```
    pub fn set_capacity(&self, capacity_in_bytes: usize) {
        let mut cache = self.0.lock().unwrap();

        cache.capacity = capacity_in_bytes;
        cache._shrink_to_fit(0); // resize cache to fit new size
    }

    /// Add a new object to the cache.
    ///
    /// If the key already exists the existing entry is replaced.
    /// Otherwise if the cache is full the least-recently-used
    /// cached objects are discarded before the new object is added.
    ///
    /// This function returns a reference to the newly added object.
    pub fn put(&self, key: K, val: V, size: usize) -> Arc<V> {
        let mut cache = self.0.lock().unwrap();

        // remove key if it exists
        if let Some(old_val) = cache.lru.remove(&key) {
            cache.total_size -= old_val.size;
        }

        // drop entries to clear cache space
        cache._shrink_to_fit(size);

        // add the new entry
        let val = Arc::new(val);
        cache.lru.insert(
            key,
            CacheItem {
                entry: val.clone(),
                size,
            },
        );
        cache.total_size += size;
        val
    }

    /// Retrieve a cached object.
    ///
    /// If the key does not exist this function returns None.
    /// Otherwise it returns a reference to the cached object.
    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        let mut cache = self.0.lock().unwrap();

        let val = cache.lru.get_refresh(key)?;
        Some(val.entry.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_capacity() {
        let cache: LruCache<u32, u32> = LruCache::new(100);
        assert_eq!(cache.get_capacity(), 100);

        cache.put(0, 0, 100);

        cache.set_capacity(0);
        assert_eq!(cache.get_capacity(), 0);

        // make sure entry was evicted
        assert!(cache.get(&0).is_none());
    }

    #[test]
    fn test_cache_empty() {
        let cache: LruCache<u32, u32> = LruCache::new(100);

        // check key that was never inserted in cache
        assert!(cache.get(&0).is_none());
    }

    #[test]
    fn test_cache_eviction() {
        let cache: LruCache<u32, u32> = LruCache::new(100);

        cache.put(0, 0, 100);
        cache.put(1, 1, 100);

        // check if first entry was evicted from cache
        assert!(cache.get(&0).is_none());

        // check if second entry still exists
        assert_eq!(cache.get(&1), Some(Arc::new(1)));
    }

    #[test]
    fn test_cache_lru() {
        let cache: LruCache<u32, u32> = LruCache::new(200);

        cache.put(0, 0, 100);
        cache.put(1, 1, 100);

        // This should bring first entry to top
        assert_eq!(cache.get(&0), Some(Arc::new(0)));

        // this should now push the second entry out of the cache
        cache.put(2, 2, 100);

        // check if second entry was evicted from cache
        assert!(cache.get(&1).is_none());

        // check if first and third entries still exists
        assert_eq!(cache.get(&0), Some(Arc::new(0)));
        assert_eq!(cache.get(&2), Some(Arc::new(2)));
    }
}

/// Wrappers around `LruCache` to provide FFI-bindings for libopenslide.
pub mod ffi {
    use super::LruCache;
    use std::hash::Hash;
    use std::os::raw::{c_int, c_void};
    use std::ptr;
    use std::sync::Arc;

    #[allow(non_camel_case_types)]
    type size_t = usize;

    // CacheKey is used to collect the parts of the key and make the function
    // signatures more readable. But it isn't really exposed through the FFI
    // API so there isn't much to document.
    #[doc(hidden)]
    #[derive(Hash, Eq, PartialEq)]
    pub struct CacheKey(*const c_void, i64, i64);

    /// A CacheEntry struct that wraps C pointers with a custom drop function.
    ///
    /// We use this so we free the cached data that was allocated with
    /// `g_slice_alloc` in the calling C code.
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
        size: size_t,
    }

    impl Drop for CacheEntry {
        fn drop(&mut self) {
            #[link(name = "glib-2.0")]
            extern "C" {
                fn g_slice_free1(size: size_t, data: *mut c_void);
            }
            unsafe {
                g_slice_free1(self.size, self.data);
            }
        }
    }

    /// Useful cache size to allocate per open slide handle.
    /// currently defaults to 32MB.
    pub const _OPENSLIDE_USEFUL_CACHE_SIZE: size_t = 1024 * 1024 * 32;

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
                drop(Box::from_raw(cache));
            };
        }
    }

    /// Get the currently configured maximum cache size.
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
            },
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
                drop(Arc::from_raw(entry));
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
                        fn g_slice_alloc(size: size_t) -> *mut c_void;
                    }
                    let size = 100 * 1024 * 1024;
                    let data = g_slice_alloc(size);
                    _openslide_cache_put(cache, null, i, 0, data, size as c_int, null_ptr);
                }
            }

            _openslide_cache_destroy(cache);
        }
    }
}
