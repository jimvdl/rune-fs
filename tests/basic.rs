#[cfg(test)]
mod osrs {
    use runefs::Dat2;
    use runefs::{Index, Indices};
    use std::collections::HashMap;

    #[test]
    fn new_indices() {
        let _indices = Indices::new("./data/osrs_cache").unwrap();
    }

    #[test]
    fn new_dat2() {
        let _dat2 = Dat2::new("./data/osrs_cache/main_file_cache.dat2").unwrap();
    }

    #[test]
    fn correct_layout() {
        let mut map: HashMap<u8, u8> = (0..=20).into_iter().map(|i| (i, i)).collect();
        map.insert(255, 255);

        let indices: HashMap<u8, u8> = Indices::new("./data/osrs_cache")
            .unwrap()
            .into_iter()
            .map(|(k, i)| (k, i.id))
            .collect();

        assert_eq!(map, indices);
    }

    #[test]
    fn from_path_correct_extension() {
        let index2 = Index::from_path(2, "./data/osrs_cache/main_file_cache.idx2").unwrap();
        let index15 = Index::from_path(15, "./data/osrs_cache/main_file_cache.idx15").unwrap();
        let index255 = Index::from_path(255, "./data/osrs_cache/main_file_cache.idx255").unwrap();

        assert_eq!(index2.id, 2);
        assert_eq!(index15.id, 15);
        assert_eq!(index255.id, 255);
    }

    #[test]
    #[should_panic]
    fn from_path_incorrect_extension() {
        Index::from_path(2, "../data/osrs_cache/main_file_cache.idx1").unwrap();
    }
}

#[cfg(all(test, feature = "rs3"))]
mod rs3 {
    use runefs::Dat2;
    use runefs::{Index, Indices};
    use std::collections::HashMap;

    #[test]
    fn new_indices() {
        let _indices = Indices::new("./data/rs3_cache").unwrap();
    }

    #[test]
    fn new_dat2() {
        let _dat2 = Dat2::new("./data/rs3_cache/main_file_cache.dat2").unwrap();
    }

    #[test]
    fn correct_layout() {
        let mut map: HashMap<u8, u8> = (0..=56).into_iter().map(|i| (i, i)).collect();
        map.insert(255, 255);

        let indices: HashMap<u8, u8> = Indices::new("./data/rs3_cache")
            .unwrap()
            .into_iter()
            .map(|(k, i)| (k, i.id))
            .collect();

        assert_eq!(map, indices);
    }

    #[test]
    fn from_path_correct_extension() {
        let index2 = Index::from_path(2, "./data/rs3_cache/main_file_cache.idx2").unwrap();
        let index15 = Index::from_path(15, "./data/rs3_cache/main_file_cache.idx15").unwrap();
        let index255 = Index::from_path(255, "./data/rs3_cache/main_file_cache.idx255").unwrap();

        assert_eq!(index2.id, 2);
        assert_eq!(index15.id, 15);
        assert_eq!(index255.id, 255);
    }

    #[test]
    #[should_panic]
    fn from_path_incorrect_extension() {
        Index::from_path(2, "../data/rs3_cache/main_file_cache.idx1").unwrap();
    }
}
