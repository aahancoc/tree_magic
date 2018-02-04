//! Handles "base types" such as inode/* and text/plain
extern crate std;

const TYPES: [&'static str; 5] =
[
    "all/all",
    "all/allfiles",
    "inode/directory",
    "text/plain",
    "application/octet-stream"
];

/// Hold metadata in cache
pub type Cache = std::fs::Metadata;

pub mod init {

    extern crate fnv;
    use fnv::FnvHashMap;
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
    
    pub fn get_aliaslist() -> FnvHashMap<MIME, MIME> {
        FnvHashMap::default()
    }
    
}

pub mod check {

    extern crate std;
    extern crate parking_lot;
    use std::path::Path;
    use super::super::{Cache, CacheItem, slurp_to_cache};
    
    /// If there are any null bytes, return False. Otherwise return True.
    fn is_text_plain_from_u8(b: &[u8]) -> bool {
        b.iter().filter(|&x| *x == 0).count() == 0
    }

    // TODO: Hoist the main logic here somewhere else. This'll get redundant fast!
    fn is_text_plain_from_filepath(filepath: &Path, filecache: &CacheItem) -> bool {
    
        let b = match slurp_to_cache(filepath, filecache, 512) {
            Ok(x) => x,
            Err(_) => return false
        };
        is_text_plain_from_u8(b.as_slice())
    }
    
    #[allow(unused_variables)]
    pub fn from_u8(
        b: &[u8], mimetype: &str, cache: &CacheItem, filecache: &CacheItem
    ) -> bool {
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
    
    pub fn from_filepath(
        filepath: &Path, mimetype: &str, cache: &CacheItem, filecache: &CacheItem
    ) -> bool{
    
        use std::fs;
        //assert_eq!(0, std::sync::Arc::strong_count(&filecache));
        
        if cache.read().is_none() {
            // Being bad with error handling here,
            // but if you can't open it it's probably not a file.
            let mut meta = cache.write();
            *meta = match fs::metadata(filepath) {
                Ok(x) => Some(Cache::Basetype(x)),
                Err(_) => {return false;}
            };
        }
        let meta = match cache.read().clone().unwrap() {
            Cache::Basetype(x) => {x},
            _ => {panic!("Invalid cache type (must be basetype)!");}
        };
        
        match mimetype {
            "all/all" => return true,
            "all/allfiles" | "application/octet-stream" => return meta.is_file(),
            "inode/directory" => return meta.is_dir(),
            "text/plain" => return is_text_plain_from_filepath(filepath, filecache),
            _ => return false
        }
    }
}
