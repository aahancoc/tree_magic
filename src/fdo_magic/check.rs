use petgraph::prelude::*;
use crate::{MIME};

fn from_u8_singlerule(file: &[u8], rule: &super::MagicRule) -> bool {
	
	// Check if we're even in bounds
	let bound_min =
		rule.start_off as usize;
	let bound_max =
			rule.start_off as usize +
			rule.val_len as usize +
			rule.region_len as usize;

	if (file.len()) < bound_max {
		return false;
	}
	
	if rule.region_len == 0 {
		
		//println!("Region == 0");
		
		match rule.mask {
			None => {
				//println!("\tMask == None");
				let x: Vec<u8> = file.iter().skip(bound_min).take(bound_max - bound_min).map(|&x| x).collect();
				//println!("\t{:?} / {:?}", x, rule.val);
				//println!("\tIndent: {}, Start: {}", rule.indent_level, rule.start_off);
				return rule.val.iter().eq(x.iter());
			},
			Some(ref mask) => {
				//println!("\tMask == Some, len == {}", mask.len());
				//println!("\tIndent: {}, Start: {}", rule.indent_level, rule.start_off);
				let mut x: Vec<u8> = file.iter()
					.skip(bound_min) // Skip to start of area
					.take(bound_max - bound_min) // Take until end of area - region length
					.map(|&x| x).collect(); // Convert to vector
				let mut val: Vec<u8> = rule.val.iter().map(|&x| x).collect();
				//println!("\t{:?} / {:?}", x, rule.val);
				
				
				assert_eq!(x.len(), mask.len());
				for i in 0..std::cmp::min(x.len(), mask.len()) {
					x[i] &= mask[i];
					val[i] = val[i] & mask[i];
				}
				//println!("\t & {:?} => {:?}", mask, x);
				
				return rule.val.iter().eq(x.iter());
			}
		}
	
	} else {
		//println!("\tRegion == {}", rule.region_len);
		//println!("\tIndent: {}, Start: {}", rule.indent_level, rule.start_off);
				
		// Define our testing slice
		let ref x: Vec<u8> = file.iter().take(file.len()).map(|&x| x).collect();
		let testarea: Vec<u8> = x.iter().skip(bound_min).take(bound_max - bound_min).map(|&x| x).collect();
		//println!("{:?}, {:?}, {:?}\n", file, testarea, rule.val);
		
		// Search down until we find a hit
		let mut y = Vec::<u8>::with_capacity(testarea.len());
		for x in testarea.windows(rule.val_len as usize) {

			y.clear();
			
			// Apply mask to value
			let ref rule_mask = rule.mask;
			match *rule_mask {
				Some(ref mask) => {

					for i in 0..rule.val_len {
						y.push(x[i as usize] & mask[i as usize]);
					}
				},
				None => y = x.to_vec(),
			}
		
			if y.iter().eq(rule.val.iter()) {
				return true;
			}
		}
	}

	false
}

/// Test every given rule by walking graph
/// TODO: Not loving the code duplication here.
pub fn from_u8_walker(
	file: &[u8],
	mimetype: MIME,
	graph: &DiGraph<super::MagicRule, u32>,
	node: NodeIndex,
	isroot: bool
) -> bool {

	let n = graph.neighbors_directed(node, Outgoing);
	
	if isroot {
		let ref rule = graph[node];
		
		// Check root
		if !from_u8_singlerule(&file, rule) {
			return false;
		}
		
		// Return if that was the only test
		if n.clone().count() == 0 {
			return true;
		}
		
		// Otherwise next indent level is lower, so continue
	}
	
	// Check subrules recursively
	for y in n {
		let ref rule = graph[y];
		
		if from_u8_singlerule(&file, rule) {
			// Check next indent level if needed
			if graph.neighbors_directed(y, Outgoing).count() != 0 {
				return from_u8_walker(file, mimetype, graph, y, false);
			// Next indent level is lower, so this must be it
			} else {
				return true;
			}
		}
	}
	
	false
}