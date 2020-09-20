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

//! LRU Cache that evicts objects based on sum of object sizes

extern crate linked_hash_map;
use linked_hash_map::LinkedHashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

struct CacheItem<V> {
    entry: Arc<V>,
    size: usize,
}

struct _LruCache<K, V> {
    lru: LinkedHashMap<K, CacheItem<V>>,
    capacity: usize,
    total_size: usize,
}

impl<K, V> _LruCache<K, V>
where
    K: Hash + Eq,
{
    fn new(capacity_in_bytes: usize) -> _LruCache<K, V> {
        _LruCache {
            lru: LinkedHashMap::new(),
            capacity: capacity_in_bytes,
            total_size: 0,
        }
    }

    fn _shrink_to_fit(&mut self, reserve: usize) {
        // drop entries to clear cache space
        while self.total_size + reserve > self.capacity {
            match self.lru.pop_front() {
                Some(val) => {
                    self.total_size -= val.1.size;
                }
                None => break,
            }
        }
    }

    fn get_capacity(&self) -> usize {
        self.capacity
    }

    fn set_capacity(&mut self, capacity_in_bytes: usize) {
        self.capacity = capacity_in_bytes;
        self._shrink_to_fit(0); // resize cache to fit new size
    }

    fn put(&mut self, key: K, val: V, size: usize) -> Arc<V> {
        // remove key if it exists
        if let Some(old_val) = self.lru.remove(&key) {
            self.total_size -= old_val.size;
        }

        // drop entries to clear cache space
        self._shrink_to_fit(size);

        // add the new entry
        let val = Arc::new(val);
        self.lru.insert(
            key,
            CacheItem {
                entry: val.clone(),
                size,
            },
        );
        self.total_size += size;
        val
    }

    fn get(&mut self, key: &K) -> Option<Arc<V>> {
        match self.lru.get_refresh(key) {
            Some(val) => Some(val.entry.clone()),
            None => None,
        }
    }
}

pub struct LruCache<K, V>(Mutex<_LruCache<K, V>>);

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq,
{
    pub fn new(capacity_in_bytes: usize) -> LruCache<K, V> {
        LruCache(Mutex::new(_LruCache::new(capacity_in_bytes)))
    }

    pub fn get_capacity(&self) -> usize {
        let cache = self.0.lock().unwrap();
        cache.get_capacity()
    }

    pub fn set_capacity(&self, capacity_in_bytes: usize) {
        let mut cache = self.0.lock().unwrap();
        cache.set_capacity(capacity_in_bytes);
    }

    pub fn put(&self, key: K, val: V, size: usize) -> Arc<V> {
        let mut cache = self.0.lock().unwrap();
        cache.put(key, val, size)
    }

    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        let mut cache = self.0.lock().unwrap();
        cache.get(key)
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
