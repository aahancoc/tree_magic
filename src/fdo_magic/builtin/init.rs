use fnv::FnvHashMap;
use crate::MIME;

/// Read all subclass lines from file
fn read_subclasses() -> Result<Vec<(MIME, MIME)>, std::io::Error> {

	let r = include_str!("subclasses");
	let mut subclasses = Vec::<(MIME, MIME)>::new();
	
	for line in r.lines() {
		let child = line.split_whitespace().nth(0).unwrap_or("").parse().unwrap_or(mime::APPLICATION_OCTET_STREAM);
		let parent = line.split_whitespace().nth(1).unwrap_or("").parse().unwrap_or(mime::APPLICATION_OCTET_STREAM);
		
		subclasses.push( (parent, child) );
	}
	
	Ok(subclasses)
}

// Get filetype aliases
fn read_aliaslist() -> Result<FnvHashMap<MIME, MIME>, std::io::Error> {
	let raliases = include_str!("aliases");
	let mut aliaslist = FnvHashMap::<MIME, MIME>::default();
	
	for line in raliases.lines() {
		let a = line.split_whitespace().nth(0).unwrap_or("").parse().unwrap_or(mime::APPLICATION_OCTET_STREAM);
		let b = line.split_whitespace().nth(1).unwrap_or("").parse().unwrap_or(mime::APPLICATION_OCTET_STREAM);
		aliaslist.insert(a,b);
	}
	
	let aliaslist = aliaslist;
	Ok(aliaslist)
}

pub fn get_aliaslist() -> FnvHashMap<MIME, MIME> {
	read_aliaslist().unwrap_or(FnvHashMap::default())
}

/// Get list of supported MIME types
pub fn get_supported() -> Vec<MIME> {
	super::ALLRULES.keys().cloned().collect()
}

/// Get list of parent -> child subclass links
pub fn get_subclasses() -> Vec<(MIME, MIME)> {

	let mut subclasses = read_subclasses().unwrap_or(Vec::<(MIME, MIME)>::new());
	
	// If child or parent refers to an alias, change it to the real type
	for x in 0..subclasses.len(){
		match super::ALIASES.get(&subclasses[x].0) {
			Some(alias) => {subclasses[x].0 = alias.clone();}
			None => {}
		}
		match super::ALIASES.get(&subclasses[x].1) {
			Some(alias) => {subclasses[x].1 = alias.clone();}
			None => {}
		}
	}
	
	subclasses
}