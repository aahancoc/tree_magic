mod from_u8 {

    extern crate tree_magic;

    macro_rules! convmime {
        ($x:expr) => {$x.parse().unwrap()}
    }

    ///Image tests
    #[test]
    fn image_gif() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/gif")),
            convmime!("image/gif")
        );
    }
    #[test]
    fn image_png() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/png")),
            convmime!("image/png")
        );
    }
    #[test]
	// GNU file reports image/x-ms-bmp
    fn image_bmp() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/bmp")),
            convmime!("image/bmp")
        );
    }
    #[test]
    fn image_tiff() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/tiff")),
            convmime!("image/tiff")
        );
    }
    #[test]
    fn image_x_portable_bitmap() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/x-portable-bitmap")),
            convmime!("image/x-portable-bitmap")
        );
    }
    #[test]
    fn image_x_pcx() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/x-pcx")),
            convmime!("image/vnd.zbrush.pcx")
        );
    }
    #[test]
    fn image_x_tga() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("image/x-tga")),
            convmime!("image/x-tga")
        );
    }


    /// Archive tests
    #[test]
    fn application_tar() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("application/x-tar")),
            convmime!("application/x-tar")
        );
    }
    #[test]
    fn application_x_7z() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("application/x-7z-compressed")),
            convmime!("application/x-7z-compressed")
        );
    }
    #[test]
    fn application_zip() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("application/zip")),
            convmime!("application/zip")
        );
    }

    /// Text tests
    #[test]
    fn text_plain() {
        assert_eq!(
            tree_magic::from_u8(include_bytes!("text/plain")),
            convmime!("text/plain")
        );
    }

	// Audio tests
	#[test]
	fn audio_flac() {
		assert_eq!(
            tree_magic::from_u8(include_bytes!("audio/flac")),
            convmime!("audio/flac")
        );
	}
	
	#[test]
	fn audio_mpeg() {
		assert_eq!(
            tree_magic::from_u8(include_bytes!("audio/mpeg")),
            convmime!("audio/mpeg")
        );
	}
	
}
