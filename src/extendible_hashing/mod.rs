mod bucket_page {
    use std::{
        fmt::{Debug, Display},
        hash::Hash,
    };

    #[derive(Clone, Debug)]
    pub(crate) struct Node<K, V>
    where
        K: Eq + Display + Hash + Clone + Debug,
        V: Display + Clone + Debug,
    {
        pub key: K,

        pub value: V,

        pub hash_code: usize,
    }

    pub(crate) const BUCKET_DEFAULT_INIT_DEPTH: usize = 2;

    #[derive(Debug, Clone)]
    pub(crate) struct BucketPage<K, V>
    where
        K: Eq + Display + Hash + Clone + Debug,
        V: Display + Clone + Debug,
    {
        pub depth: usize,

        pub size: usize,

        pub elems: Vec<Option<Node<K, V>>>,
    }

    impl<K, V> Default for BucketPage<K, V>
    where
        K: Eq + Display + Hash + Clone + Debug,
        V: Display + Clone + Debug,
    {
        fn default() -> Self {
            Self::new(BUCKET_DEFAULT_INIT_DEPTH)
        }
    }

    impl<K, V> BucketPage<K, V>
    where
        K: Eq + Display + Hash + Clone + Debug,
        V: Display + Clone + Debug,
    {
        pub fn new(depth: usize) -> Self {
            Self {
                depth,
                size: 0,
                elems: vec![None; 1 << depth],
            }
        }

        pub fn put(&mut self, key: K, value: V, hash_code: usize) -> Result<(), (K, V, usize)> {
            let mut insert_index: Option<usize> = None;
            for (index, elem) in self.elems.iter_mut().enumerate() {
                if let Some(elem) = elem {
                    if hash_code == elem.hash_code && key == elem.key {
                        elem.value = value;
                        return Ok(());
                    }
                } else {
                    if insert_index.is_none() {
                        insert_index = Some(index);
                    }
                }
            }

            if let Some(insert_index) = insert_index {
                self.elems[insert_index] = Some(Node {
                    key,
                    value,
                    hash_code,
                });
                self.size += 1;
                return Ok(());
            }
            Err((key, value, hash_code))
        }

        pub fn del(&mut self, key: &K, hash_code: usize) -> Option<Node<K, V>> {
            for opt_elem in self.elems.iter_mut() {
                if let Some(elem) = opt_elem {
                    if hash_code == elem.hash_code && *key == elem.key {
                        self.size -= 1;
                        return opt_elem.take();
                    }
                }
            }
            None
        }

        pub fn get(&self, key: &K, hash_code: usize) -> Option<&V> {
            for elem in self.elems.iter() {
                if let Some(elem) = elem {
                    if hash_code == elem.hash_code && *key == elem.key {
                        return Some(&elem.value);
                    }
                }
            }
            None
        }

        pub fn grow(&mut self) {
            for _ in 0..(1 << self.depth) {
                self.elems.push(None);
            }
            self.depth += 1;
        }

        pub fn shrink(&mut self) {
            self.depth -= 1;
            let mut new_elems = vec![None; 1 << self.depth];
            let mut index = 0;
            for opt_elem in self.elems.iter_mut() {
                if opt_elem.is_some() {
                    new_elems[index] = opt_elem.take();
                    index += 1;
                }
            }

            self.elems = new_elems;
        }

        pub fn contain(&self, key: &K, hash_code: usize) -> bool {
            for elem in self.elems.iter() {
                if let Some(elem) = elem {
                    if hash_code == elem.hash_code && *key == elem.key {
                        return true;
                    }
                }
            }
            false
        }
    }
}

mod directory_page {
    use crate::extendible_hashing::bucket_page::BUCKET_DEFAULT_INIT_DEPTH;
    use std::{
        cell::RefCell,
        fmt::{Debug, Display},
        hash::Hash,
        rc::Rc,
    };

    use super::bucket_page::{BucketPage, Node};

    pub(crate) const DIRECTORY_DEFAULT_INIT_GLOBAL_DEPTH: usize = 3;

    const DIRECTORY_MERGE_LOAD_FACTOR_BIT: usize = 3;

    const DIRECTORY_SHRINK_LOAD_FACTOR: f32 = 0.25;

    const _: () = {
        if DIRECTORY_MERGE_LOAD_FACTOR_BIT < (1 << 1) {
            panic!("DIRECTORY_MERGE_LOAD_FACTOR_BIT must be greater than 2(1<<1)");
        }
    };

    #[derive(Debug)]
    pub(crate) struct DirectoryPage<K, V>
    where
        K: Eq + Display + Hash + Clone + Debug,
        V: Display + Clone + Debug,
    {
        pub global_depth: usize,

        pub buckets: Vec<Rc<RefCell<BucketPage<K, V>>>>,

        pub size: usize,
    }

    impl<K, V> Default for DirectoryPage<K, V>
    where
        K: Eq + Display + Hash + Clone + Debug,
        V: Display + Clone + Debug,
    {
        fn default() -> Self {
            Self::new(DIRECTORY_DEFAULT_INIT_GLOBAL_DEPTH)
        }
    }

    impl<K, V> DirectoryPage<K, V>
    where
        K: Eq + Display + Hash + Clone + Debug,
        V: Display + Clone + Debug,
    {
        pub fn new(global_depth: usize) -> Self {
            let bucket_page: Rc<RefCell<BucketPage<K, V>>> = Rc::new(RefCell::new(
                BucketPage::new(std::cmp::min(BUCKET_DEFAULT_INIT_DEPTH, global_depth)),
            ));
            Self {
                global_depth,
                buckets: vec![bucket_page; 1 << global_depth],
                size: 0,
            }
        }

        pub fn put(&mut self, key: K, value: V, hash_code: usize) {
            let mut directory_index = self.get_directory_index(hash_code);
            let res = self.buckets[directory_index]
                .borrow_mut()
                .put(key, value, hash_code);

            match res {
                Ok(_) => {}
                Err((k, v, h)) => {
                    self.split(directory_index);
                    directory_index = self.get_directory_index(hash_code);
                    let _ = self.buckets[directory_index].borrow_mut().put(k, v, h);
                }
            }
            self.size += 1;
        }

        pub fn get(&self, key: &K, hash_code: usize) -> Option<V> {
            let directory_index = self.get_directory_index(hash_code);
            let bucket = self.buckets[directory_index].borrow();
            let res = bucket.get(key, hash_code);
            match res {
                Some(value) => Some(value.clone()),
                None => None,
            }
        }

        pub fn del(&mut self, key: &K, hash_code: usize) -> Option<(K, V)> {
            let directory_index = self.get_directory_index(hash_code);
            let res = {
                let mut bucket = self.buckets[directory_index].borrow_mut();
                bucket.del(key, hash_code)
            };

            match res {
                Some(node) => {
                    self.size -= 1;
                    self.try_merge(directory_index);

                    if self.size
                        < (self.global_depth as f32 * DIRECTORY_SHRINK_LOAD_FACTOR as f32) as usize
                    {
                        self.try_shrink();
                    }
                    Some((node.key, node.value))
                }
                None => None,
            }
        }

        pub fn contain(&self, key: &K, hash_code: usize) -> bool {
            let directory_index = self.get_directory_index(hash_code);
            let bucket = self.buckets[directory_index].borrow();
            bucket.contain(key, hash_code)
        }

        fn get_directory_index(&self, hash_code: usize) -> usize {
            hash_code as usize & ((1 << self.global_depth) - 1)
        }

        fn pair_index(bucket_no: usize, local_depth: usize) -> usize {
            bucket_no ^ (1 << (local_depth - 1))
        }

        fn grow(&mut self) {
            for i in 0..(1 << self.global_depth) {
                self.buckets.push(self.buckets[i].clone());
            }
            self.global_depth += 1;
        }

        fn can_shrink(&self) -> bool {
            for bucket in self.buckets.iter() {
                if bucket.borrow().depth == self.global_depth {
                    return false;
                }
            }
            true
        }

        fn try_shrink(&mut self) {
            if !self.can_shrink() {
                return;
            }
            self.global_depth -= 1;
            for _ in 0..(1 << self.global_depth) {
                self.buckets.pop();
            }
        }

        fn split(&mut self, bucket_no: usize) {
            let bucket = self.buckets[bucket_no].clone();
            bucket.borrow_mut().grow();

            let new_local_depth = bucket.borrow().depth;
            if new_local_depth > self.global_depth {
                self.grow();
            }

            let pair_index = Self::pair_index(bucket_no, new_local_depth);
            self.buckets[pair_index] = Rc::new(RefCell::new(BucketPage::new(new_local_depth)));

            let mask = (1 << new_local_depth) - 1;

            let mut remove_count = 0;
            for opt_elem in bucket.borrow_mut().elems.iter_mut() {
                if let Some(elem) = opt_elem {
                    if elem.hash_code as usize & mask == pair_index & mask {
                        // need to move
                        let Node {
                            key,
                            value,
                            hash_code,
                        } = opt_elem.take().unwrap();
                        let _ = self.buckets[pair_index]
                            .borrow_mut()
                            .put(key, value, hash_code);
                        remove_count += 1;
                    }
                }
            }
            bucket.borrow_mut().size -= remove_count;

            let old_bucket = bucket;
            let new_bucket = self.buckets[pair_index].clone();

            for (index, bucket) in self.buckets.iter_mut().enumerate() {
                if index & mask == bucket_no & mask {
                    *bucket = old_bucket.clone();
                }
                if index & mask == pair_index & mask {
                    *bucket = new_bucket.clone();
                }
            }
        }

        fn try_merge(&mut self, bucket_no: usize) {
            let local_depth = self.buckets[bucket_no].borrow().depth;

            if local_depth <= DIRECTORY_DEFAULT_INIT_GLOBAL_DEPTH {
                return;
            }

            let pair_index = Self::pair_index(bucket_no, local_depth);
            let pair_index_local_path = self.buckets[pair_index].borrow().depth;
            let size = self.buckets[bucket_no].borrow().size;
            let pair_index_size = self.buckets[pair_index].borrow().size;

            if local_depth == pair_index_local_path
                && (size << DIRECTORY_MERGE_LOAD_FACTOR_BIT) < (1 << local_depth)
                && (pair_index_size << DIRECTORY_MERGE_LOAD_FACTOR_BIT)
                    < (1 << pair_index_local_path)
            {
                let mut elems = Vec::with_capacity(pair_index_size);
                for opt_elem in self.buckets[pair_index].borrow_mut().elems.iter_mut() {
                    if opt_elem.is_some() {
                        let Node {
                            key,
                            value,
                            hash_code,
                        } = opt_elem.take().unwrap();
                        elems.push((key, value, hash_code));
                    }
                }

                {
                    let mut buckets = self.buckets[bucket_no].borrow_mut();
                    elems.into_iter().for_each(|(k, v, h)| {
                        let _ = buckets.put(k, v, h);
                    });
                }

                self.buckets[bucket_no].borrow_mut().size += pair_index_size;
                self.buckets[bucket_no].borrow_mut().shrink();

                let new_bucket = self.buckets[bucket_no].clone();
                let mask = (1 << local_depth) - 1;

                for (index, bucket) in self.buckets.iter_mut().enumerate() {
                    if index & mask == pair_index & mask && bucket.borrow().depth == local_depth {
                        *bucket = new_bucket.clone();
                    }
                }
            }
        }
    }
}

use std::{
    fmt::{Debug, Display},
    hash::{DefaultHasher, Hash, Hasher},
};

use directory_page::*;

pub const EXTENDIBLEHASHING_DEFAULT_DEPTH: usize = 10;

#[derive(Debug)]
pub struct ExtendibleHashing<K, V>
where
    K: Eq + Display + Hash + Clone + Debug,
    V: Display + Clone + Debug,
{
    directory_pages: Vec<Option<DirectoryPage<K, V>>>,

    depth: usize,

    size: usize,
}

impl<K, V> Default for ExtendibleHashing<K, V>
where
    K: Eq + Display + Hash + Clone + Debug,
    V: Display + Clone + Debug,
{
    fn default() -> Self {
        Self::new(EXTENDIBLEHASHING_DEFAULT_DEPTH)
    }
}

impl<K, V> ExtendibleHashing<K, V>
where
    K: Eq + Display + Hash + Clone + Debug,
    V: Display + Clone + Debug,
{
    pub fn new(depth: usize) -> Self {
        if depth as u32 > usize::BITS {
            panic!("depth > bits of usize!")
        }

        let mut directory_pages = Vec::new();
        for _ in 0..(1 << depth) {
            directory_pages.push(None);
        }
        Self {
            depth,
            directory_pages,
            size: 0,
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        let hash_code = Self::hash_code(&key);
        let directory_pages_index = hash_code >> (usize::BITS - self.depth as u32);
        match &mut self.directory_pages[directory_pages_index] {
            Some(page) => {
                // If there is a page, insert the key-value pair into it
                let old_size = page.size;
                page.put(key, value, hash_code);
                self.size += page.size - old_size;
            }
            None => {
                // If there is no page, allocate a new page and insert the key-value pair into it
                let mut new_page = DirectoryPage::default();
                new_page.put(key, value, hash_code);
                self.directory_pages[directory_pages_index] = Some(new_page);
                self.size += 1;
            }
        }
    }

    pub fn contain(&self, key: &K) -> bool {
        let hash_code = Self::hash_code(key);
        let directory_pages_index = hash_code >> (usize::BITS - self.depth as u32);
        match &self.directory_pages[directory_pages_index] {
            Some(page) => page.contain(key, hash_code),
            None => false,
        }
    }

    pub fn del(&mut self, key: &K) -> Option<(K, V)> {
        let hash_code = Self::hash_code(key);
        let directory_pages_index = hash_code >> (usize::BITS - self.depth as u32);
        match &mut self.directory_pages[directory_pages_index] {
            Some(page) => {
                let res = page.del(key, hash_code);
                match res {
                    Some(_) => {
                        self.size -= 1;
                        res
                    }
                    None => None,
                }
            }
            None => None,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let hash_code = Self::hash_code(key);
        let directory_pages_index = hash_code >> (usize::BITS - self.depth as u32);
        match &self.directory_pages[directory_pages_index] {
            Some(page) => page.get(key, hash_code),
            None => None,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[inline]
    pub fn depth(&self) -> usize {
        self.depth
    }

    fn hash_code(key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize
    }
}

#[cfg(test)]
mod bucket_page_test {
    use super::bucket_page::*;
    use std::hash::Hasher;
    use std::{
        fmt::{Debug, Display},
        hash::{DefaultHasher, Hash},
    };

    fn test_hash_code<K>(key: &K) -> usize
    where
        K: Eq + Display + Hash + Clone + Debug,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize
    }

    #[test]
    fn test_bucket_page_put() {
        let mut bucket_page1: BucketPage<String, String> = BucketPage::default();
        let key1 = String::from("key1");
        let value1 = String::from("value1");
        let hash_code1 = test_hash_code(&key1);
        let _ = bucket_page1.put(key1, value1, hash_code1);
        assert_eq!(bucket_page1.size, 1);

        let mut bucket_page2: BucketPage<String, String> = BucketPage::default();
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = test_hash_code(&key);
            assert!(bucket_page2.put(key, value, hash_code).is_ok());
        }
        assert_eq!(bucket_page2.size, 1 << BUCKET_DEFAULT_INIT_DEPTH);
        let key = format!("key");
        let value = format!("value");
        let hash_code = test_hash_code(&key);
        assert!(!bucket_page2.put(key, value, hash_code).is_ok());
    }

    #[test]
    fn test_bucket_page_get() {
        let mut bucket_page: BucketPage<String, String> = BucketPage::default();
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = test_hash_code(&key);
            assert!(bucket_page.put(key, value, hash_code).is_ok());
        }
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = test_hash_code(&key);
            assert_eq!(bucket_page.get(&key, hash_code), Some(&value));
        }

        for i in (1 << BUCKET_DEFAULT_INIT_DEPTH)..(1 << (BUCKET_DEFAULT_INIT_DEPTH + 1)) {
            let key = format!("key{}", i + 1);
            let hash_code = test_hash_code(&key);
            assert_eq!(bucket_page.get(&key, hash_code), None);
        }
    }

    #[test]
    fn test_bucket_page_del() {
        let mut bucket_page: BucketPage<String, String> = BucketPage::default();
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = test_hash_code(&key);
            assert!(bucket_page.put(key, value, hash_code).is_ok());
        }

        let key = format!("key1");
        let value = format!("value1");
        let hash_code = test_hash_code(&key);
        let del_value = bucket_page.del(&key, hash_code);
        assert_eq!(del_value.clone().unwrap().key, key);
        assert_eq!(del_value.clone().unwrap().value, value);
        assert_eq!(&del_value.unwrap().hash_code, &hash_code);

        assert_eq!(bucket_page.size, (1 << BUCKET_DEFAULT_INIT_DEPTH) - 1);
        assert_eq!(
            bucket_page.get(&format!("key1"), test_hash_code(&format!("key1")),),
            None
        );
    }

    #[test]
    fn test_bucket_page_grow() {
        let mut bucket_page: BucketPage<String, String> = BucketPage::default();
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = test_hash_code(&key);
            assert!(bucket_page.put(key, value, hash_code).is_ok());
        }
        bucket_page.grow();

        assert_eq!(bucket_page.depth, BUCKET_DEFAULT_INIT_DEPTH + 1);

        for i in (1 << BUCKET_DEFAULT_INIT_DEPTH)..(1 << (BUCKET_DEFAULT_INIT_DEPTH + 1)) {
            assert!(bucket_page.elems[i].is_none());
        }
    }

    #[test]
    fn test_bucket_page_contain() {
        let mut bucket_page: BucketPage<String, String> = BucketPage::default();
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = test_hash_code(&key);
            assert!(bucket_page.put(key, value, hash_code).is_ok());
        }
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let hash_code = test_hash_code(&key);
            assert!(bucket_page.contain(&key, hash_code));
        }

        for i in (1 << BUCKET_DEFAULT_INIT_DEPTH)..(1 << (BUCKET_DEFAULT_INIT_DEPTH + 1)) {
            let key = format!("key{}", i + 1);
            let hash_code = test_hash_code(&key);
            assert!(!bucket_page.contain(&key, hash_code));
        }
    }
}

#[cfg(test)]
mod directory_page_test {
    use crate::extendible_hashing::bucket_page::BUCKET_DEFAULT_INIT_DEPTH;
    use std::hash::Hasher;
    use std::{
        fmt::{Debug, Display},
        hash::{DefaultHasher, Hash},
    };

    use super::directory_page::*;

    fn test_hash_code<K>(key: &K) -> usize
    where
        K: Eq + Display + Hash + Clone + Debug,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize
    }

    #[test]
    fn test_directory_page_default_new() {
        let directory_page: DirectoryPage<String, String> = DirectoryPage::default();
        assert_eq!(directory_page.size, 0);
        assert_eq!(
            directory_page.global_depth,
            DIRECTORY_DEFAULT_INIT_GLOBAL_DEPTH
        );

        for bucket in directory_page.buckets.iter() {
            assert_eq!(bucket.borrow().depth, BUCKET_DEFAULT_INIT_DEPTH);
        }

        let directory_page: DirectoryPage<String, String> = DirectoryPage::new(4);
        assert_eq!(directory_page.size, 0);
        assert_eq!(directory_page.global_depth, 4);

        for bucket in directory_page.buckets.iter() {
            assert_eq!(bucket.borrow().depth, BUCKET_DEFAULT_INIT_DEPTH);
        }

        let directory_page: DirectoryPage<String, String> = DirectoryPage::new(1);
        assert_eq!(directory_page.size, 0);
        assert_eq!(directory_page.global_depth, 1);

        for bucket in directory_page.buckets.iter() {
            assert_eq!(bucket.borrow().depth, 1);
        }
    }

    #[test]
    fn test_directory_page_put_len_and_get() {
        let mut directory_page: DirectoryPage<String, String> = DirectoryPage::default();
        directory_page.put(
            format!("key"),
            format!("value"),
            test_hash_code(&format!("key")),
        );
        assert_eq!(
            directory_page.get(&format!("key"), test_hash_code(&format!("key"))),
            Some(format!("value"))
        );

        let mut directory_page: DirectoryPage<String, String> = DirectoryPage::default();
        for i in 0..10000 {
            directory_page.put(
                format!("key{}", i + 1),
                format!("value{}", i + 1),
                test_hash_code(&format!("key{}", i + 1)),
            );
        }
        assert_eq!(directory_page.size, 10000);
        for i in 0..10000 {
            assert_eq!(
                directory_page.get(
                    &format!("key{}", i + 1),
                    test_hash_code(&format!("key{}", i + 1))
                ),
                Some(format!("value{}", i + 1))
            );
        }
        for i in 10000..20000 {
            assert_eq!(
                directory_page.get(
                    &format!("key{}", i + 1),
                    test_hash_code(&format!("key{}", i + 1))
                ),
                None
            );
        }
    }

    #[test]
    fn test_directory_page_contain() {
        let mut directory_page: DirectoryPage<String, String> = DirectoryPage::default();
        directory_page.put(
            format!("key"),
            format!("value"),
            test_hash_code(&format!("key")),
        );
        assert!(directory_page.contain(&format!("key"), test_hash_code(&format!("key"))));

        let mut directory_page: DirectoryPage<String, String> = DirectoryPage::default();
        for i in 0..10000 {
            directory_page.put(
                format!("key{}", i + 1),
                format!("value{}", i + 1),
                test_hash_code(&format!("key{}", i + 1)),
            );
        }
        for i in 0..10000 {
            assert!(directory_page.contain(
                &format!("key{}", i + 1),
                test_hash_code(&format!("key{}", i + 1))
            ));
        }
        for i in 10000..20000 {
            assert!(!directory_page.contain(
                &format!("key{}", i + 1),
                test_hash_code(&format!("key{}", i + 1))
            ));
        }
    }

    #[test]
    fn test_directory_page_del() {
        let mut directory_page: DirectoryPage<String, String> = DirectoryPage::default();
        directory_page.put(
            format!("key"),
            format!("value"),
            test_hash_code(&format!("key")),
        );
        assert!(directory_page.contain(&format!("key"), test_hash_code(&format!("key"))));
        assert_eq!(
            directory_page.del(&format!("key"), test_hash_code(&format!("key"))),
            Some((format!("key"), format!("value")))
        );
        assert_eq!(directory_page.size, 0);
        assert!(directory_page.size == 0);
        assert!(!directory_page.contain(&format!("key"), test_hash_code(&format!("key"))));

        let mut directory_page: DirectoryPage<String, String> = DirectoryPage::default();
        for i in 0..10000 {
            directory_page.put(
                format!("key{}", i + 1),
                format!("value{}", i + 1),
                test_hash_code(&format!("key{}", i + 1)),
            );
        }

        for i in 0..10000 {
            assert_eq!(
                directory_page.del(
                    &format!("key{}", i + 1),
                    test_hash_code(&format!("key{}", i + 1))
                ),
                Some((format!("key{}", i + 1), format!("value{}", i + 1)))
            );
            assert_eq!(directory_page.size, 10000 - i - 1);
            assert!(!directory_page.contain(
                &format!("key{}", i + 1),
                test_hash_code(&format!("key{}", i + 1))
            ));
        }
        assert!(directory_page.size == 0);
    }
}
