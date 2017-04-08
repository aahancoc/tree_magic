extern crate nom;
extern crate std;
use nom::*;
use std::str;

#[derive(Debug)]
pub struct MagicRules {
    pub indent_level: u32,
    pub start_off: u32,
    pub val_len: u16,
    pub val: Vec<u8>,
    pub mask: Option<Vec<u8>>,
    pub word_len: u32,
    pub region_len: u32
}

#[derive(Debug)]
pub struct MagicEntry {
    pub mime: String,
    pub rules: Vec<MagicRules>
}

// Below functions from https://github.com/badboy/iso8601/blob/master/src/helper.rs
// but modified to be safe and provide defaults
use std::str::{FromStr, from_utf8_unchecked};

pub fn to_string(s: &[u8]) -> std::result::Result<&str, std::str::Utf8Error> {
    str::from_utf8(s)
}
pub fn to_u32(s: std::result::Result<&str, std::str::Utf8Error>, def: u32) -> u32 {
    //
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

#[test]
fn mime_test() {
    assert_eq!(mime(&b"[90:text/plain]\n"[..]), IResult::Done(&b""[..], "text/plain"));
}


// Indent levels sub-parser for magic_rules
// Default value 0
named!(magic_rules_indent_level<u32>,
    do_parse!(
        ret: take_until!(">") >> 
        (buf_to_u32(ret, 0))
    )
);

#[test]
fn indent_level_test() {
    assert_eq!(magic_rules_indent_level(&b"0>fgh"[..]).to_result().unwrap(), 0);
    assert_eq!(magic_rules_indent_level(&b"42>fgh"[..]).to_result().unwrap(), 42);
    assert_eq!(magic_rules_indent_level(&b">fgh"[..]).to_result().unwrap(), 0);
}

// Start offset sub-parser for magic_rules
named!(magic_rules_start_off<u32>,
    do_parse!(
        ret: take_until!("=") >>
        (buf_to_u32(ret, 0))
    )
);

#[test]
fn start_off_test() {
    assert_eq!(magic_rules_start_off(&b"0="[..]).to_result().unwrap(), 0);
    assert_eq!(magic_rules_start_off(&b"42="[..]).to_result().unwrap(), 42);
}

// Singular magic ruleset
named!(magic_rules<MagicRules>,
    do_parse!(
        peek!(is_not!("[")) >>
    
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
                (buf_to_u32(ret, 1))
            )
        ) >>
        
        (MagicRules{
            indent_level: _indent_level,
            start_off: _start_off,
            val: _val,
            val_len: _val_len,
            mask: _mask,
            word_len: _word_len.unwrap_or(1),
            region_len: _region_len.unwrap_or(1)
        })
    )
);

// Singular magic entry
named!(magic_entry<MagicEntry>,
    do_parse!(
        _mime: do_parse!(
            ret: mime >>
            (ret.to_string())
        ) >>
        
        _rules: many0!(magic_rules) >>
    
        (MagicEntry{
            mime: _mime,
            rules: _rules
        })
    )
);

// Magic file
named!(pub magic_file<Vec<MagicEntry>>,
    do_parse!(
        tag!("MIME-Magic\0\n") >>
        ret: many0!(magic_entry) >>
        
        (ret)
    )
);
