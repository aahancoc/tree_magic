// Common routines for all fdo_magic parsers

extern crate std;
extern crate petgraph;
extern crate fnv;

pub mod builtin;
#[cfg(not(feature="staticmime"))]
pub mod sys;

// We can't have staticmime and sys_fdo_magic enabled
// because we can't statically refer to a file on disk.
#[cfg(all(feature="staticmime", all(feature="sys_fdo_magic", unix)))]
const CONF_ERROR_CANNOT_USE_STATICMIME_WITH_SYS_FDO_MAGIC: u32 = ();

#[derive(Debug, Clone)]
pub struct MagicRule {
    pub indent_level: u32,
    pub start_off: u32,
    pub val_len: u16,
    pub val: Vec<u8>,
    pub mask: Option<Vec<u8>>,
    pub word_len: u32,
    pub region_len: u32
}

#[cfg(not(feature="staticmime"))]
macro_rules! convmime {
    ($x:expr) => {$x.to_string()}
}
#[cfg(feature="staticmime")]
macro_rules! convmime {
    ($x:expr) => {$x}
}

pub mod ruleset {
    extern crate nom;
    extern crate std;
	extern crate petgraph;
	extern crate fnv;
    use std::str;
	use petgraph::prelude::*;
	use fnv::FnvHashMap;
    use MIME;

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
    #[cfg(not(feature="staticmime"))]
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
    #[cfg(feature="staticmime")]
    named!(mime<&'static str>,
        do_parse!(
            res: delimited!(
                delimited!(
                    char!('['),
                    is_not!(":"),
                    char!(':')
                ),
                is_not!("]"), // the mime
                tag!("]\n") 
            ) >>
            // Yes I am aware that this is horribly dangerous
            // but there is no reason this shouldn't be fine
            // because the source is static and known and really
            // a string is just a slice of u8s isn't it?
            (unsafe{
                std::mem::transmute(res)
            })
        )
    );
    
    #[test]
    // Ensures the transmute used in mime for feature="staticmime"
    // doesn't blow up.
    fn str_transmute_sanity() {
        unsafe {
            const A: &'static [u8] = b"Hello world!";
            const B: &'static str = "Hello world!";
            let c: &'static str = std::mem::transmute(A);
            assert!(B == c); // 256
        }
    }

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
            _mime: do_parse!(
                ret: mime >>
                (convmime!(ret))
            ) >>
            _rules: many0!(magic_rules) >>
            (_mime, _rules)
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
			rulestack.push( (x.clone(), xnode) );
			
		}
		
		let graph = graph;
		graph
	}
    
    pub fn from_u8(b: &[u8]) -> Result<FnvHashMap<MIME, DiGraph<super::MagicRule, u32>>, String> {
        let tuplevec = from_u8_to_tuple_vec(b).to_result().map_err(|e| e.to_string())?;;
        let mut res = FnvHashMap::<MIME, DiGraph<super::MagicRule, u32>>::default();
        
        for x in tuplevec {
            res.insert(x.0, gen_graph(x.1));
        }
        
        Ok(res)
        
    }

    /// Loads the given magic file and outputs a vector of MagicEntry structs
    #[cfg(not(feature="staticmime"))]
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

}

// Functions to check if a file matches a magic entry
pub mod check {

    extern crate std;
    extern crate petgraph;
    use petgraph::prelude::*;
    
    fn from_u8_singlerule(file: &[u8], rule: &super::MagicRule) -> bool {
        
        // Check if we're even in bounds
        let bound_min = //std::cmp::min(
            rule.start_off as usize;
            //rule.val.len()
        //);
        let bound_max =
            //std::cmp::min(
            //(
                rule.start_off as usize +
                rule.val_len as usize +
                rule.region_len as usize;
            //),
            //rule.val.len()
        //);

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
					//let mut val: Vec<u8> = rule.val.iter().map(|&x| x).collect();
					//println!("\t{:?} / {:?}", x, rule.val);
					
					
					assert_eq!(x.len(), mask.len());
					for i in 0..std::cmp::min(x.len(), mask.len()) {
						x[i] &= mask[i];
						//val[i] = val[i] & mask[i];
					}
					//println!("\t & {:?} => {:?}", mask, x);
					
					return rule.val.iter().eq(x.iter());
				}
			}
		
		} else {
			//println!("\tRegion == {}", rule.region_len);
			//println!("\t{:?} / {:?}", x, rule.val);
			//println!("\tIndent: {}, Start: {}", rule.indent_level, rule.start_off);
					
			// Define our testing slice
			let ref x: Vec<u8> = file.iter().take(file.len()).map(|&x| x).collect();
			let testarea: Vec<u8> = x.iter().skip(bound_min).take(bound_max - bound_min).map(|&x| x).collect();
			//let testarea: Vec<u8> = file.iter().skip(bound_min).take(bound_max - bound_min).map(|&x| x).collect();
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
        mimetype: &str,
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

}

