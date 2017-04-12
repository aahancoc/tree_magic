mod from_u8 {
    #![feature(test)]
    extern crate test;
    use self::test::Bencher;
    extern crate tree_magic;

    ///Image tests
    #[bench]
    fn image_gif(b: &mut Bencher) {
        b.iter(|| tree_magic::from_u8(include_bytes!("image/gif")));
    }
    #[bench]
    fn image_png(b: &mut Bencher) {
        b.iter(|| tree_magic::from_u8(include_bytes!("image/png")));
    }
    
    /// Archive tests
    #[bench]
    fn application_zip(b: &mut Bencher) {
        b.iter(|| tree_magic::from_u8(include_bytes!("application/zip")));
    }

    /// Text tests
    #[bench]
    fn test_plain(b: &mut Bencher) {
        b.iter(|| tree_magic::from_u8(include_bytes!("text/plain")));
    }

}
