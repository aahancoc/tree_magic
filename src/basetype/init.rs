use fnv::FnvHashMap;
use crate::MIME;

pub fn get_supported() -> Vec<MIME> {
super::TYPES.to_vec().iter().map(|x| x.parse().unwrap()).collect()
}

/// Returns Vec of parent->child relations
pub fn get_subclasses() -> Vec<(MIME, MIME)> {
let mut res = Vec::<(MIME, MIME)>::new();

// There's probably a better way to do this.
res.push( ("all/all".parse().unwrap(),                  "all/allfiles".parse().unwrap()) );
res.push( ("all/all".parse().unwrap(),                  "inode/directory".parse().unwrap()) );
res.push( ("all/allfiles".parse().unwrap(),             "application/octet-stream".parse().unwrap()) );
res.push( ("application/octet-stream".parse().unwrap(), "text/plain".parse().unwrap()) );

res
}

pub fn get_aliaslist() -> FnvHashMap<MIME, MIME> {
FnvHashMap::default()
}