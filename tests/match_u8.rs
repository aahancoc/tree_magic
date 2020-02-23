mod match_u8 {
    extern crate tree_magic;

    ///Image tests
    #[test]
    fn image_gif() {
        assert!(tree_magic::match_u8("image/gif", include_bytes!("image/gif")));
    }
    #[test]
    fn image_png() {
        assert!(tree_magic::match_u8("image/png", include_bytes!("image/png")));
    }
    #[test]
	// GNU file reports as image/x-ms-bmp
    fn image_x_bmp() {
        assert!(tree_magic::match_u8("image/bmp", include_bytes!("image/bmp")));
    }
    #[test]
    fn image_tiff() {
        assert!(tree_magic::match_u8("image/tiff", include_bytes!("image/tiff")));
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
    
	// Audio tests
	#[test]
    fn audio_flac() {
        assert!(tree_magic::match_u8("audio/flac", include_bytes!("audio/flac")));
    }
	#[test]
    fn audio_mpeg() {
        assert!(tree_magic::match_u8("audio/mpeg", include_bytes!("audio/mpeg")));
    }
	#[test]
    fn audio_ogg() {
        assert!(tree_magic::match_u8("audio/ogg", include_bytes!("audio/ogg")));
    }
	#[test]
    fn audio_wav() {
        assert!(tree_magic::match_u8("audio/wav", include_bytes!("audio/wav")));
    }
}
