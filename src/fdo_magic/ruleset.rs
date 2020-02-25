use std::str;
use petgraph::prelude::*;
use fnv::FnvHashMap;
use crate::MIME;

// Below functions from https://github.com/badboy/iso8601/blob/master/src/helper.rs
// but modified to be safe and provide defaults
pub fn to_string(s: &[u8]) -> std::result::Result<&str, std::str::Utf8Error> {
	str::from_utf8(s)
}
pub fn to_u32(s: std::result::Result<&str, std::str::Utf8Error>, def: u32) -> u32 {
	
	match s {
		Ok (t) => {str::FromStr::from_str(t).unwrap_or(def)},
		Err (_) => def
	}
}

pub fn buf_to_u32(s: &[u8], def: u32) -> u32 {
	to_u32(to_string(s), def)
}

// Initial mime string
// Format: [priority: mime]   
named!(mime<&str>,
	map_res!(
		delimited!(
			delimited!(
				char!('['),
				is_not!(":"),
				char!(':')
			),
			is_not!("]"), // the mime
			tag!("]\n") 
		),
		str::from_utf8
	)
);

// Indent levels sub-parser for magic_rules
// Default value 0
named!(magic_rules_indent_level<u32>,
	do_parse!(
		ret: take_until!(">") >> 
		(buf_to_u32(ret, 0))
	)
);

// Start offset sub-parser for magic_rules
named!(magic_rules_start_off<u32>,
	do_parse!(
		ret: take_until!("=") >>
		(buf_to_u32(ret, 0))
	)
);

// Singular magic ruleset
named!(magic_rules<super::MagicRule>,
	do_parse!(
		peek!(is_a!("012345689>")) >>
		_indent_level: magic_rules_indent_level >>
		tag!(">") >>
		_start_off: magic_rules_start_off >>
		tag!("=") >>
		_val_len: u16!(nom::Endianness::Big) >> // length of value
		_val: do_parse!(
			ret: take!(_val_len) >>
			(ret.iter().map(|&x| x).collect())
		) >> // value
		
		_mask: opt!(
			do_parse!(
				char!('&') >>
				ret: take!(_val_len) >> // mask (default 0xFF)
				(ret.iter().map(|&x| x).collect())
			)
		) >>
		
		// word size (default 1)
		_word_len: opt!(
			do_parse!(
				tag!("~") >>
				ret: take_until!("+") >>
				(buf_to_u32(ret, 1))
			)
		) >>
		
		// length of region in file to check (default 1)
		_region_len: opt!(
			do_parse!(
				tag!("+") >>
				ret: take_until!("\n") >>
				(buf_to_u32(ret, 0))
			)
		) >>
		
		take_until_and_consume!("\n") >>
		
		(super::MagicRule{
			indent_level: _indent_level,
			start_off: _start_off,
			val: _val,
			val_len: _val_len,
			mask: _mask,
			word_len: _word_len.unwrap_or(1),
			region_len: _region_len.unwrap_or(0)
		})
	)
	
);

/// Singular magic entry
named!(magic_entry<(MIME, Vec<super::MagicRule>)>,
	do_parse!(
		_mime: do_parse!(ret: mime >> (ret.parse().unwrap_or(mime::APPLICATION_OCTET_STREAM))) >>
		_rules: many0!(magic_rules) >> (_mime, _rules)
	)
);

/// Converts a magic file given as a &[u8] array
/// to a vector of MagicEntry structs
named!(from_u8_to_tuple_vec<Vec<(MIME, Vec<super::MagicRule>)>>,
	do_parse!(
		tag!("MIME-Magic\0\n") >>
		ret: many0!(magic_entry) >>
		(ret)
	)
);

fn gen_graph(magic_rules: Vec<super::MagicRule>) -> DiGraph<super::MagicRule, u32>
{
	use petgraph::prelude::*;
	// Whip up a graph real quick
	let mut graph = DiGraph::<super::MagicRule, u32>::new();
	let mut rulestack = Vec::<(super::MagicRule, NodeIndex)>::new();
	
	for x in magic_rules {
		let xnode = graph.add_node(x.clone());
		
		loop {
			let y = rulestack.pop();
			match y {
				None => {break;},
				Some(rule) => {
					if rule.0.indent_level < x.indent_level {
						graph.add_edge(rule.1, xnode, 1);
						rulestack.push( rule );
						break;
					}
				}
			};
		}
		rulestack.push( (x, xnode) );
		
	}
	
	let graph = graph;
	graph
}

pub fn from_u8(b: &[u8]) -> Result<FnvHashMap<MIME, DiGraph<super::MagicRule, u32>>, String> {
	let tuplevec = from_u8_to_tuple_vec(b).to_result().map_err(|e| e.to_string())?;
	let mut res = FnvHashMap::<MIME, DiGraph<super::MagicRule, u32>>::default();
	
	for x in tuplevec {
		res.insert(x.0, gen_graph(x.1));
	}
	
	Ok(res)
	
}

/// Loads the given magic file and outputs a vector of MagicEntry structs
pub fn from_filepath(filepath: &str) -> Result<FnvHashMap<MIME, DiGraph<super::MagicRule, u32>>, String>{
	use std::io::prelude::*;
	use std::io::BufReader;
	use std::fs::File;

	let fmagic = File::open(filepath).map_err(|e| e.to_string())?;
	let mut rmagic = BufReader::new(fmagic);
	let mut bmagic = Vec::<u8>::new();
	rmagic.read_to_end(&mut bmagic).map_err(|e| e.to_string())?;
	
	let magic_ruleset = from_u8(
		bmagic.as_slice()
	).map_err(|e| e.to_string())?;
	
	Ok(magic_ruleset)
}
