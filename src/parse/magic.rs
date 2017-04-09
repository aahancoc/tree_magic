extern crate std;

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

pub mod ruleset {
    extern crate nom;
    extern crate std;
    use std::str;
    use nom::*;
    use std::collections::HashMap;
    
    // This is the catch-all when all else fails
    // (but it's not that bad.)
    pub fn can_check(mime: String) -> bool {
        true
    }

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
        assert_eq!(magic_rules_indent_level(&b"xyz>fgh"[..]).to_result().is_err(), true);
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
                    (buf_to_u32(ret, 1))
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
                region_len: _region_len.unwrap_or(1)
            })
        )
        
    );

    // Singular magic entry
    named!(magic_entry<(String, Vec<super::MagicRule>)>,
        do_parse!(
            _mime: do_parse!(
                ret: mime >>
                (ret.to_string())
            ) >>
            
            _rules: many0!(magic_rules) >>
        
            (_mime, _rules)
        )
    );

    /// Converts a magic file given as a &[u8] array
    /// to a vector of MagicEntry structs
    named!(from_u8_to_tuple_vec<Vec<(String, Vec<super::MagicRule>)>>,
        do_parse!(
            tag!("MIME-Magic\0\n") >>
            ret: many0!(magic_entry) >>
            (ret)
        )
    );
    
    pub fn from_u8(b: &[u8]) -> Result<HashMap<String, Vec<super::MagicRule>>, String> {
        let tuplevec = from_u8_to_tuple_vec(b).to_result().map_err(|e| e.to_string())?;
        let mut res = HashMap::<String, Vec<super::MagicRule>>::new();
        
        for x in tuplevec {
            res.insert(x.0, x.1);
        }
        
        Ok(res)
        
    }

    /// Loads the given magic file and outputs a vector of MagicEntry structs
    pub fn from_filepath(filepath: &str) -> Result<HashMap<String, Vec<super::MagicRule>>, String>{
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
pub mod test{

    extern crate std;
    
    fn from_vec_u8_singlerule(file: &Vec<u8>, rule: super::MagicRule) -> bool {
        
        // Check if we're even in bounds
        let bound_min = std::cmp::min(
            rule.start_off as usize,
            rule.val.iter().count()
        );
        let bound_max =
            std::cmp::min(
            (
                rule.start_off as usize +
                rule.val_len as usize +
                rule.region_len as usize
            ),
            rule.val.iter().count()
        );

        if file.iter().count() < bound_max {
            return false;
        }
        
        // Define our testing slice
        let ref x: Vec<u8> = *file;
        let testarea: Vec<u8> = x[bound_min .. bound_max].to_vec();
        //println!("{:?}, {:?}", file, testarea);
        
        // Search down until we find a hit
        for x in testarea.windows(rule.val_len as usize) {
        
            // Apply mask to value
            let mut y: Vec<u8>;
            
            match rule.mask.clone() {
                Some(mask) => {
                    y = Vec::<u8>::new();
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

        false
    }
    
    pub fn can_check(mimetype: &str) -> bool {
        true
    }

    /// Test against all rules
    pub fn from_vec_u8(file: Vec<u8>, mimetype: &str, magic_rules: Vec<super::MagicRule>) -> bool {
    
        for x in magic_rules {
                match from_vec_u8_singlerule(&file, x) {
                    true => return true,
                    false => continue,
                }
        }
        
        false
    }
    
    pub fn from_filepath(filepath: &str, mimetype: &str, magic_rules: Vec<super::MagicRule>) -> Result<bool, std::io::Error>{
        use std::io::prelude::*;
        use std::io::BufReader;
        use std::fs::File;

        // Get # of bytes to read
        let mut scanlen:u64 = 0;
        for x in &magic_rules {
            let tmplen:u64 = 
                x.start_off as u64 +
                x.val_len as u64 +
                x.region_len as u64;
                
            if tmplen > scanlen {
                scanlen = tmplen;
            }
        }
        
        let f = File::open(filepath)?;
        let r = BufReader::new(f);
        let mut b = Vec::<u8>::new();
        r.take(scanlen).read_to_end(&mut b)?;
        
        Ok(from_vec_u8(b, mimetype, magic_rules))
    }

}

