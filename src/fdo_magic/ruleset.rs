use std::str;
use petgraph::prelude::*;
use fnv::FnvHashMap;
use crate::MIME;
use nom::{
  IResult,
  bytes::complete::{is_not, tag, take, take_while},
  character::is_digit,
  combinator::{map, map_res, opt},
  multi::many0,
  number::complete::be_u16,
  sequence::{delimited, preceded, terminated, tuple},
};

// Singular magic ruleset
fn magic_rules(input: &[u8]) -> IResult<&[u8], super::MagicRule> {
    let int_or = |default| map(
        take_while(is_digit),
        move |digits| str::from_utf8(digits).unwrap().parse().unwrap_or(default)
    );

    let (input, (indent_level, start_off, val_len)) = tuple((
        terminated(int_or(0), tag(">")),
        terminated(int_or(0), tag("=")),
        be_u16,
    ))(input)?;

    let (input, (val, mask, word_len, region_len)) = terminated(
        tuple((
            take(val_len),
            opt(preceded(tag("&"), take(val_len))),
            opt(preceded(tag("~"), int_or(1))),
            opt(preceded(tag("+"), int_or(0))),
        )),
        tag("\n")
    )(input)?;

    Ok((input, super::MagicRule {
        indent_level,
        start_off,
        val: val.to_vec(),
        val_len,
        mask: mask.map(Vec::from),
        word_len: word_len.unwrap_or(1),
        region_len: region_len.unwrap_or(0),
    }))
}

/// Converts a magic file given as a &[u8] array
/// to a vector of MagicEntry structs
fn ruleset(input: &[u8]) -> IResult<&[u8], Vec<(MIME, Vec<super::MagicRule>)>> {
    // Parse the MIME type from "[priority: mime]"
    let mime = map(map_res(
        terminated(
            delimited(
                delimited(tag("["), is_not(":"), tag(":")), // priority
                is_not("]"), // mime
                tag("]")
            ),
            tag("\n"),
        ),
        str::from_utf8),
    str::to_string);

    let magic_entry = tuple((mime, many0(magic_rules)));
    preceded(tag("MIME-Magic\0\n"), many0(magic_entry))(input)
}

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
	let tuplevec = ruleset(b).map_err(|e| e.to_string())?.1;
	let res = tuplevec.into_iter().map(|x| (x.0, gen_graph(x.1))).collect();
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
