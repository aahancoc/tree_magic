mod match_u8 {
    extern crate tree_magic;

    #[cfg(not(feature="staticmime"))]
    macro_rules! convmime {
        ($x:expr) => {$x.to_string()}
    }
    #[cfg(feature="staticmime")]
    macro_rules! convmime {
        ($x:expr) => {$x}
    }

    ///Image tests
    #[test]
    fn image_gif() {
        assert!(tree_magic::match_u8("image/gif", include_bytes!("image/gif")));
    }
    #[bench]
    fn bench_image_gif(b: &mut Bencher) {
        b.iter(|| tree_magic::match_u8("image/gif", include_bytes!("image/gif")));
    }
    #[test]
    fn image_png() {
        assert!(tree_magic::match_u8("image/png", include_bytes!("image/png")));
    }
    #[test]
    fn image_x_ms_bmp() {
        assert!(tree_magic::match_u8("image/x-ms-bmp", include_bytes!("image/x-ms-bmp")));
    }
    #[test]
    fn image_tiff() {
        assert!(tree_magic::match_u8("image/tiff", include_bytes!("image/tiff")));
    }
    #[test]
    fn image_x_lss16() {
        assert!(tree_magic::match_u8("image/x-lss16", include_bytes!("image/x-lss16")));
    }
    #[test]
    fn image_x_portable_pixmap() {
        assert!(tree_magic::match_u8("image/x-portable-pixmap", include_bytes!("image/x-portable-pixmap")));
    }
    #[test]
    fn image_x_portable_bitmap() {
        assert!(tree_magic::match_u8("image/x-portable-bitmap", include_bytes!("image/x-portable-bitmap")));
    }
    #[test]
    fn image_x_pcx() {
        assert!(tree_magic::match_u8("image/x-pcx", include_bytes!("image/x-pcx")));
    }
    #[test]
    // GNU file reports this as image/x-xpmi
    fn image_x_xpixmap() {
        assert!(tree_magic::match_u8("image/x-xpixmap", include_bytes!("image/x-xpixmap")));
    }
    #[test]
    fn image_x_tga() {
        assert!(tree_magic::match_u8("image/x-tga", include_bytes!("image/x-tga")));
    }


    /// Archive tests
    #[test]
    fn application_tar() {
        assert!(tree_magic::match_u8("application/x-tar", include_bytes!("application/x-tar")));
    }
    #[test]
    fn application_x_7z() {
        assert!(tree_magic::match_u8("application/x-7z-compressed", include_bytes!("application/x-7z-compressed")));
    }
    #[test]
    fn application_zip() {
        assert!(tree_magic::match_u8("application/zip", include_bytes!("application/zip")));
    }

    /// Text tests
    #[test]
    fn text_plain() {
        assert!(tree_magic::match_u8("text/plain", include_bytes!("text/plain")));
    }
    

}
