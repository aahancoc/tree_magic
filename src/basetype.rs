//! Handles "base types" such as inode/* and text/plain

#[cfg(feature="staticmime")] type MIME = &'static str;
#[cfg(not(feature="staticmime"))] type MIME = String;

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

pub mod test {

    extern crate std;
    
    /// If there are any null bytes, return False. Otherwise return True.
    fn is_text_plain_from_u8(b: &[u8]) -> bool {
        b.iter().filter(|&x| *x == 0).count() == 0
    }

    fn is_text_plain_from_filepath(filepath: &str) -> Result<bool, std::io::Error> {
        use std::io::prelude::*;
        use std::io::BufReader;
        use std::fs::File;
        
        // Slurp up first 1024 (or less) bytes
        let f = File::open(filepath)?;
        let r = BufReader::new(f);
        let mut b = Vec::<u8>::new();
        r.take(512).read_to_end(&mut b)?;
        
        Ok(is_text_plain_from_u8(b.as_slice()))
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

    pub fn can_check(mime: &str) -> bool {
        return super::TYPES.contains(&mime);
    }
    
    pub fn from_filepath(filepath: &str, mimetype: &str) -> Result<bool, std::io::Error>{
    
        use std::fs;
        let meta = fs::metadata(filepath)?;
        
        match mimetype {
            "all/all" => return Ok(true),
            "all/allfiles" => return Ok(meta.is_file()),
            "inode/directory" => return Ok(meta.is_dir()),
            "text/plain" => return is_text_plain_from_filepath(filepath),
            "application/octet-stream" => return Ok(meta.is_file()),
            _ => {
                println!("{}", mimetype);
                panic!("This mime is not supported by the mod. (See can_check)")
            }
        }
        
    }
}
