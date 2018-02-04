mod from_filepath {

    extern crate tree_magic;

    use std::path::Path;

    #[test]
    fn nonexistent_file_returns_none() {
        assert_eq!(
            tree_magic::from_filepath(Path::new("this/file/does/not/exist")),
            None
        );
    }

}