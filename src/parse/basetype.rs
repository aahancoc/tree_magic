// Handles "base types" such as inode/* and text/plain

const TYPES: [&'static str; 5] =
[
    "all/all",
    "all/allfiles",
    "inode/directory",
    "text/plain",
    "application/octet-stream"
];

pub mod test {

    extern crate std;
    
    /// If there are any null bytes, return False. Otherwise return True.
    fn is_text_plain_from_vec_u8(b: Vec<u8>) -> bool {
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
        r.take(1024).read_to_end(&mut b)?;
        
        Ok(is_text_plain_from_vec_u8(b))
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
