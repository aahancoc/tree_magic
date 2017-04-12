mod from_u8 {

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
    fn from_u8_image_gif() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/gif")),
            Some(convmime!("image/gif"))
        );
    }
    #[test]
    fn from_u8_image_png() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/png")),
            Some(convmime!("image/png"))
        );
    }
    #[test]
    fn image_x_ms_bmp() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/x-ms-bmp")),
            Some(convmime!("imageimage_x_ms_bmp/x-ms-bmp"))
        );
    }
    #[test]
    fn image_tiff() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/tiff")),
            Some(convmime!("image/tiff"))
        );
    }
    #[test]
    fn image_x_lss16() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/x-lss16")),
            Some(convmime!("image/x-lss16"))
        );
    }
    #[test]
    fn image_x_portable_pixmap() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/x-portable-pixmap")),
            Some(convmime!("image/x-portable-pixmap"))
        );
    }
    #[test]
    fn image_x_portable_bitmap() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/x-portable-bitmap")),
            Some(convmime!("image/x-portable-bitmap"))
        );
    }
    #[test]
    fn image_x_pcx() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/x-pcx")),
            Some(convmime!("image/x-pcx"))
        );
    }
    #[test]
    // GNU file reports this as image/x-xpmi
    fn image_x_xpixmap() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/x-xpixmap")),
            Some(convmime!("image/x-xpixmap"))
        );
    }
    #[test]
    fn image_x_tga() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/x-tga")),
            Some(convmime!("image/x-tga"))
        );
    }


    /// Archive tests
    #[test]
    fn application_tar() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("application/x-tar")),
            Some(convmime!("application/x-tar"))
        );
    }
    #[test]
    fn application_x_7z() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("application/x-7z-compressed")),
            Some(convmime!("application/x-7z-compressed"))
        );
    }
    #[test]
    fn application_zip() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("application/zip")),
            Some(convmime!("application/zip"))
        );
    }

    /// Text tests
    #[test]
    fn text_plain() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("text/plain")),
            Some(convmime!("text/plain"))
        );
    }

}
