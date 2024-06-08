use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

#[derive(Clone, Debug)]
struct Node<K, V>
where
    K: Eq + Display + Hash + Clone + Debug,
    V: Display + Clone + Debug,
{
    key: K,
    value: V,
    hash_code: u64,
}

const BUCKET_DEFAULT_INIT_DEPTH: usize = 2;

#[derive(Debug, Clone)]
struct BucketPage<K, V>
where
    K: Eq + Display + Hash + Clone + Debug,
    V: Display + Clone + Debug,
{
    depth: usize,

    size: usize,

    elems: Vec<Option<Node<K, V>>>,
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
    fn new(depth: usize) -> Self {
        Self {
            depth,
            size: 0,
            elems: vec![None; 1 << depth],
        }
    }

    fn is_full(&self) -> bool {
        self.size == (1 << self.depth)
    }

    fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn put(&mut self, key: K, value: V, hash_code: u64) -> Result<(), (K, V, u64)> {
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

    fn del(&mut self, key: &K, hash_code: u64) -> Option<Node<K, V>> {
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

    fn get(&self, key: &K, hash_code: u64) -> Option<&V> {
        for elem in self.elems.iter() {
            if let Some(elem) = elem {
                if hash_code == elem.hash_code && *key == elem.key {
                    return Some(&elem.value);
                }
            }
        }
        None
    }

    fn get_mut(&mut self, key: &K, hash_code: u64) -> Option<&mut V> {
        for elem in self.elems.iter_mut() {
            if let Some(elem) = elem {
                if hash_code == elem.hash_code && *key == elem.key {
                    return Some(&mut elem.value);
                }
            }
        }
        None
    }

    fn clear(&mut self) {
        self.elems.iter_mut().for_each(|x| *x = None);
        self.size = 0;
    }

    fn grow(&mut self) {
        for _ in 0..(1 << self.depth) {
            self.elems.push(None);
        }
        self.depth += 1;
    }

    fn contain(&self, key: &K, hash_code: u64) -> bool {
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

const DIRECTORY_DEFAULT_INIT_GLOBAL_DEPTH: usize = 3;

struct DirectoryPage<K, V>
where
    K: Eq + Display + Hash + Clone + Debug,
    V: Display + Clone + Debug,
{
    global_depth: usize,

    buckets: Vec<Rc<RefCell<BucketPage<K, V>>>>,

    size: usize,
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
        let bucket_page: Rc<RefCell<BucketPage<K, V>>> = Rc::new(RefCell::new(BucketPage::new(
            std::cmp::min(BUCKET_DEFAULT_INIT_DEPTH, global_depth),
        )));
        Self {
            global_depth,
            buckets: vec![bucket_page; 1 << global_depth],
            size: 0,
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        let hash_code = Self::hash_code(&key);
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

    pub fn get(&self, key: &K) -> Option<V> {
        let hash_code = Self::hash_code(key);
        let directory_index = self.get_directory_index(hash_code);
        let bucket = self.buckets[directory_index].borrow();
        let res = bucket.get(key, hash_code);
        match res {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    pub fn del(&mut self, key: &K) -> Option<(K, V)> {
        let hash_code = Self::hash_code(key);
        let directory_index = self.get_directory_index(hash_code);
        let mut bucket = self.buckets[directory_index].borrow_mut();
        match bucket.del(key, hash_code) {
            Some(node) => {
                self.size -= 1;
                Some((node.key, node.value))
            }
            None => None,
        }
    }

    fn contain(&self, key: &K) -> bool {
        let hash_code = Self::hash_code(key);
        let directory_index = self.get_directory_index(hash_code);
        let bucket = self.buckets[directory_index].borrow();
        bucket.contain(key, hash_code)
    }

    fn hash_code(key: &K) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn get_directory_index(&self, hash_code: u64) -> usize {
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

    fn shrink(&mut self) {
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

    fn len(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod bucket_page_test {
    use super::*;

    #[test]
    fn test_bucket_page_default_and_new() {
        let bucket_page: BucketPage<String, String> = BucketPage::default();
        assert_eq!(bucket_page.depth, BUCKET_DEFAULT_INIT_DEPTH);
        assert_eq!(bucket_page.size, 0);
        assert_eq!(bucket_page.elems.len(), 1 << BUCKET_DEFAULT_INIT_DEPTH);
        for elem in bucket_page.elems.iter() {
            assert!(elem.is_none());
        }
        assert!(bucket_page.is_empty());
    }

    #[test]
    fn test_bucket_page_put() {
        let mut bucket_page1: BucketPage<String, String> = BucketPage::default();
        let key1 = String::from("key1");
        let value1 = String::from("value1");
        let hash_code1 = DirectoryPage::<String, String>::hash_code(&key1);
        let _ = bucket_page1.put(key1, value1, hash_code1);
        assert_eq!(bucket_page1.size, 1);

        let mut bucket_page2: BucketPage<String, String> = BucketPage::default();
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = DirectoryPage::<String, String>::hash_code(&key);
            assert!(bucket_page2.put(key, value, hash_code).is_ok());
        }
        assert_eq!(bucket_page2.size, 1 << BUCKET_DEFAULT_INIT_DEPTH);
        assert!(bucket_page2.is_full());
        let key = format!("key");
        let value = format!("value");
        let hash_code = DirectoryPage::<String, String>::hash_code(&key);
        assert!(!bucket_page2.put(key, value, hash_code).is_ok());
    }

    #[test]
    fn test_bucket_page_get() {
        let mut bucket_page: BucketPage<String, String> = BucketPage::default();
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = DirectoryPage::<String, String>::hash_code(&key);
            assert!(bucket_page.put(key, value, hash_code).is_ok());
        }
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let mut value = format!("value{}", i + 1);
            let hash_code = DirectoryPage::<String, String>::hash_code(&key);
            assert_eq!(bucket_page.get(&key, hash_code), Some(&value));
            assert_eq!(bucket_page.get_mut(&key, hash_code), Some(&mut value));
        }

        for i in (1 << BUCKET_DEFAULT_INIT_DEPTH)..(1 << (BUCKET_DEFAULT_INIT_DEPTH + 1)) {
            let key = format!("key{}", i + 1);
            let hash_code = DirectoryPage::<String, String>::hash_code(&key);
            assert_eq!(bucket_page.get(&key, hash_code), None);
            assert_eq!(bucket_page.get_mut(&key, hash_code), None);
        }
    }

    #[test]
    fn test_bucket_page_del() {
        let mut bucket_page: BucketPage<String, String> = BucketPage::default();
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = DirectoryPage::<String, String>::hash_code(&key);
            assert!(bucket_page.put(key, value, hash_code).is_ok());
        }

        let key = format!("key1");
        let value = format!("value1");
        let hash_code = DirectoryPage::<String, String>::hash_code(&key);
        let del_value = bucket_page.del(&key, hash_code);
        assert_eq!(del_value.clone().unwrap().key, key);
        assert_eq!(del_value.clone().unwrap().value, value);
        assert_eq!(&del_value.unwrap().hash_code, &hash_code);

        assert_eq!(bucket_page.size, (1 << BUCKET_DEFAULT_INIT_DEPTH) - 1);
        assert_eq!(
            bucket_page.get(
                &format!("key1"),
                DirectoryPage::<String, String>::hash_code(&format!("key1")),
            ),
            None
        );
        assert_eq!(
            bucket_page.get_mut(
                &format!("key1"),
                DirectoryPage::<String, String>::hash_code(&format!("key1")),
            ),
            None
        );
    }

    #[test]
    fn test_bucket_page_clear() {
        let mut bucket_page: BucketPage<String, String> = BucketPage::default();
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = DirectoryPage::<String, String>::hash_code(&key);
            assert!(bucket_page.put(key, value, hash_code).is_ok());
        }
        bucket_page.clear();
        assert!(bucket_page.is_empty());
    }

    #[test]
    fn test_bucket_page_grow() {
        let mut bucket_page: BucketPage<String, String> = BucketPage::default();
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let value = format!("value{}", i + 1);
            let hash_code = DirectoryPage::<String, String>::hash_code(&key);
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
            let hash_code = DirectoryPage::<String, String>::hash_code(&key);
            assert!(bucket_page.put(key, value, hash_code).is_ok());
        }
        for i in 0..(1 << BUCKET_DEFAULT_INIT_DEPTH) {
            let key = format!("key{}", i + 1);
            let hash_code = DirectoryPage::<String, String>::hash_code(&key);
            assert!(bucket_page.contain(&key, hash_code));
        }

        for i in (1 << BUCKET_DEFAULT_INIT_DEPTH)..(1 << (BUCKET_DEFAULT_INIT_DEPTH + 1)) {
            let key = format!("key{}", i + 1);
            let hash_code = DirectoryPage::<String, String>::hash_code(&key);
            assert!(!bucket_page.contain(&key, hash_code));
        }
    }
}

#[cfg(test)]
mod directory_page_test {
    use super::*;

    #[test]
    fn test_directory_page_default_new() {
        let directory_page: DirectoryPage<String, String> = DirectoryPage::default();
        assert_eq!(directory_page.len(), 0);
        assert_eq!(
            directory_page.global_depth,
            DIRECTORY_DEFAULT_INIT_GLOBAL_DEPTH
        );

        for bucket in directory_page.buckets.iter() {
            assert_eq!(bucket.borrow().depth, BUCKET_DEFAULT_INIT_DEPTH);
        }

        let directory_page: DirectoryPage<String, String> = DirectoryPage::new(4);
        assert_eq!(directory_page.len(), 0);
        assert_eq!(directory_page.global_depth, 4);

        for bucket in directory_page.buckets.iter() {
            assert_eq!(bucket.borrow().depth, BUCKET_DEFAULT_INIT_DEPTH);
        }

        let directory_page: DirectoryPage<String, String> = DirectoryPage::new(1);
        assert_eq!(directory_page.len(), 0);
        assert_eq!(directory_page.global_depth, 1);

        for bucket in directory_page.buckets.iter() {
            assert_eq!(bucket.borrow().depth, 1);
        }
    }

    #[test]
    fn test_directory_page_put_len_and_get() {
        let mut directory_page: DirectoryPage<String, String> = DirectoryPage::default();
        directory_page.put(format!("key"), format!("value"));
        assert_eq!(directory_page.get(&format!("key")), Some(format!("value")));

        let mut directory_page: DirectoryPage<String, String> = DirectoryPage::default();
        for i in 0..10000 {
            directory_page.put(format!("key{}", i + 1), format!("value{}", i + 1));
        }
        assert_eq!(directory_page.len(), 10000);
        for i in 0..10000 {
            assert_eq!(
                directory_page.get(&format!("key{}", i + 1)),
                Some(format!("value{}", i + 1))
            );
        }
        for i in 10000..20000 {
            assert_eq!(directory_page.get(&format!("key{}", i + 1)), None);
        }
    }
}
