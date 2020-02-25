use fnv::FnvHashMap;
use crate::MIME;

pub fn get_supported() -> Vec<MIME> {
super::TYPES.to_vec().iter().map(|x| x.to_string()).collect()
}

/// Returns Vec of parent->child relations
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