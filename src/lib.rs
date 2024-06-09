pub mod extendible_hashing;

#[cfg(test)]
mod test_extendible_hashing {
    use super::extendible_hashing::ExtendibleHashing;

    #[test]
    fn test_extendible_hashing_new() {
        let e_h: ExtendibleHashing<String, String> = ExtendibleHashing::default();
        assert_eq!(e_h.depth(), 10);
        assert_eq!(e_h.len(), 0);
        assert!(e_h.is_empty());

        let e_h: ExtendibleHashing<String, String> = ExtendibleHashing::new(5);
        assert_eq!(e_h.depth(), 5);
        assert_eq!(e_h.len(), 0);
        assert!(e_h.is_empty());
    }

    #[test]
    fn test_extendible_hashing_put_get_and_contain() {
        let mut e_h: ExtendibleHashing<String, String> = ExtendibleHashing::default();

        for i in 0..10000 {
            e_h.put(format!("key{}", i + 1), format!("value{}", i + 1));
        }

        for i in 0..10000 {
            assert_eq!(
                e_h.get(&format!("key{}", i + 1)),
                Some(format!("value{}", i + 1))
            );
        }

        for i in 10000..20000 {
            assert_eq!(e_h.get(&format!("key{}", i + 1)), None);
        }

        for i in 0..10000 {
            assert!(e_h.contain(&format!("key{}", i + 1)));
        }

        for i in 10000..20000 {
            assert!(!e_h.contain(&format!("key{}", i + 1)));
        }
    }

    #[test]
    fn test_extendible_hashing_del() {
        let mut e_h: ExtendibleHashing<String, String> = ExtendibleHashing::default();

        for i in 0..10000 {
            e_h.put(format!("key{}", i + 1), format!("value{}", i + 1));
        }

        for i in 0..10000 {
            assert_eq!(
                e_h.get(&format!("key{}", i + 1)),
                Some(format!("value{}", i + 1))
            );
        }

        for i in 10000..20000 {
            assert_eq!(e_h.get(&format!("key{}", i + 1)), None);
        }

        for i in 0..10000 {
            assert!(e_h.contain(&format!("key{}", i + 1)));
        }

        for i in 10000..20000 {
            assert!(!e_h.contain(&format!("key{}", i + 1)));
        }

        for i in 0..5000 {
            assert_eq!(
                e_h.del(&format!("key{}", i + 1)),
                Some((format!("key{}", i + 1), format!("value{}", i + 1)))
            );
        }

        for i in 0..5000 {
            assert!(!e_h.contain(&format!("key{}", i + 1)));
            assert_eq!(e_h.get(&format!("key{}", i + 1)), None);
        }

        for i in 5000..10000 {
            assert!(e_h.contain(&format!("key{}", i + 1)));
            assert_eq!(
                e_h.get(&format!("key{}", i + 1)),
                Some(format!("value{}", i + 1))
            );
        }

        for i in 0..5000 {
            e_h.put(format!("key{}", i + 1), format!("value{}", i + 1));
        }

        for i in 0..10000 {
            assert!(e_h.contain(&format!("key{}", i + 1)));
            assert_eq!(
                e_h.get(&format!("key{}", i + 1)),
                Some(format!("value{}", i + 1))
            );
        }
    }

    #[test]
    fn test_extendible_hashing_len_and_is_empty() {
        let mut e_h: ExtendibleHashing<String, String> = ExtendibleHashing::default();

        assert_eq!(e_h.len(), 0);
        assert!(e_h.is_empty());

        for i in 0..10000 {
            e_h.put(format!("key{}", i + 1), format!("value{}", i + 1));
        }

        assert_eq!(e_h.len(), 10000);
        assert!(!e_h.is_empty());

        for i in 0..5000 {
            assert_eq!(
                e_h.del(&format!("key{}", i + 1)),
                Some((format!("key{}", i + 1), format!("value{}", i + 1)))
            );
        }

        assert_eq!(e_h.len(), 5000);
        assert!(!e_h.is_empty());

        for i in 0..5000 {
            e_h.put(format!("key{}", i + 1), format!("value{}", i + 1));
        }

        assert_eq!(e_h.len(), 10000);
        assert!(!e_h.is_empty());
    }
}
