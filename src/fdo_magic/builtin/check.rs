use std::path::Path;
use petgraph::prelude::*;
use crate::{fdo_magic, read_bytes};
use mime::Mime as MIME;

/// Test against all rules
#[allow(unused_variables)]
pub fn from_u8(file: &[u8], mimetype: MIME) -> bool {

	// Get magic ruleset
	let graph = match super::ALLRULES.get(&mimetype) {
		Some(item) => item,
		None => return false // No rule for this mime
	};
	
	// Check all rulesets
	for x in graph.externals(Incoming) {
		if fdo_magic::check::from_u8_walker(file, mimetype, graph, x, true) {
			return true;
		}
	}
	
	false
}

/// This only exists for the case of a direct match_filepath call
/// and even then we could probably get rid of this...
#[allow(unused_variables)]
pub fn from_filepath(filepath: &Path, mimetype: MIME) -> bool{
	// Get magic ruleset
	let magic_rules = match super::ALLRULES.get(&mimetype) {
		Some(item) => item,
		None => return false // No rule for this mime
	};

	// Get # of bytes to read
	let mut scanlen = 0;
	for x in magic_rules.raw_nodes() {
		let ref y = x.weight;
		let tmplen = 
			y.start_off as usize +
			y.val_len as usize +
			y.region_len as usize;
			
		if tmplen > scanlen {
			scanlen = tmplen;
		}
	}
	
	let b = match read_bytes(filepath, scanlen) {
		Ok(x) => x,
		Err(_) => return false
	};
	
	from_u8(b.as_slice(), mimetype)
}