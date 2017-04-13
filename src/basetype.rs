//! Handles "base types" such as inode/* and text/plain

const TYPES: [&'static str; 5] =
[
    "all/all",
    "all/allfiles",
    "inode/directory",
    "text/plain",
    "application/octet-stream"
];

pub mod init {

    extern crate std;
    use MIME;
	
	/*pub fn get_aliases(mimetype: &str) -> Vec<MIME> {
		let _ = mimetype;
		Vec::<MIME>::new()
	}*/
    
    #[cfg(feature="staticmime")]
    pub fn get_supported() -> Vec<MIME> {
        super::TYPES.to_vec().iter().map(|x| *x).collect()
    }
    
    #[cfg(not(feature="staticmime"))]
    pub fn get_supported() -> Vec<MIME> {
        super::TYPES.to_vec().iter().map(|x| x.to_string()).collect()
    }
    
    /// Returns Vec of parent->child relations
    #[cfg(feature="staticmime")]
    pub fn get_subclasses() -> Vec<(MIME, MIME)> {
        let mut res = Vec::<(MIME, MIME)>::new();

        res.push( ("all/all", "all/allfiles") );
        res.push( ("all/all", "inode/directory") );
        res.push( ("all/allfiles", "application/octet-stream") );
        res.push( ("application/octet-stream", "text/plain") );
        
        res
    }
    
    #[cfg(not(feature="staticmime"))]
    pub fn get_subclasses() -> Vec<(MIME, MIME)> {
        let mut res = Vec::<(MIME, MIME)>::new();

        // There's probably a better way to do this.
        res.push( ("all/all".to_string(), "all/allfiles".to_string()) );
        res.push( ("all/all".to_string(), "inode/directory".to_string()) );
        res.push( ("all/allfiles".to_string(), "application/octet-stream".to_string()) );
        res.push( ("application/octet-stream".to_string(), "text/plain".to_string()) );
        
        res
    }
    
}

pub mod check {

    extern crate std;
    
    /// If there are any null bytes, return False. Otherwise return True.
    fn is_text_plain_from_u8(b: &[u8]) -> bool {
        b.iter().filter(|&x| *x == 0).count() == 0
    }

    fn is_text_plain_from_filepath(filepath: &str) -> bool {
        use std::io::prelude::*;
        use std::io::BufReader;
        use std::fs::File;
        
        // Slurp up first 512 (or less) bytes
        let f = match File::open(filepath) {
            Ok(x) => x,
            Err(_) => return false
        };
        let r = BufReader::new(f);
        let mut b = Vec::<u8>::new();
        match r.take(512).read_to_end(&mut b) {
            Ok(_) => {},
            Err(_) => return false
        }
        
        is_text_plain_from_u8(b.as_slice())
    }
    
    pub fn from_u8(b: &[u8], mimetype: &str) -> bool {
        if mimetype == "application/octet-stream" || mimetype == "all/allfiles" {
            // Both of these are the case if we have a bytestream at all
            return true;
        } if mimetype == "text/plain" {
            return is_text_plain_from_u8(b);
        } else {
            // ...how did we get bytes for this?
            return false;
        }
    }
    
    pub fn from_filepath(filepath: &str, mimetype: &str) -> bool{
    
        use std::fs;
        // Being bad with error handling here,
        // but if you can't open it it's probably not a file.
        let meta = match fs::metadata(filepath) {
            Ok(x) => x,
            Err(_) => {return false;}
        };
        
        match mimetype {
            "all/all" => return true,
            "all/allfiles" | "application/octet-stream" => return meta.is_file(),
            "inode/directory" => return meta.is_dir(),
            "text/plain" => return is_text_plain_from_filepath(filepath),
            _ => {
                println!("{}", mimetype);
                panic!("This mime is not supported by the mod. (See can_check)")
            }
        }
        
    }
}
