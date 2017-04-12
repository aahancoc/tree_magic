extern crate std;
use std::collections::HashMap;

#[cfg(feature="staticmime")] type MIME = &'static str;
#[cfg(not(feature="staticmime"))] type MIME = String;

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

/// Load magic file before anything else.
/// Sys magic always disabled on Windows.
#[cfg(all(feature="sys_fdo_magic", unix))]
lazy_static! {
    static ref ALLRULES: HashMap<MIME, Vec<MagicRule>> = {
        ruleset::from_filepath("/usr/share/mime/magic").unwrap_or(HashMap::new())
    };
}
#[cfg(not(all(feature="sys_fdo_magic", unix)))]
lazy_static! {
    static ref ALLRULES: HashMap<MIME, Vec<MagicRule>> = {
        ruleset::from_u8(include_bytes!("magic")).unwrap_or(HashMap::new())
    };
}

pub mod ruleset {
    extern crate nom;
    extern crate std;
    use std::str;
    use std::collections::HashMap;
    //use std::mem;
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
    
    pub fn from_u8(b: &[u8]) -> Result<HashMap<MIME, Vec<super::MagicRule>>, String> {
        let tuplevec = from_u8_to_tuple_vec(b).to_result().map_err(|e| e.to_string())?;;
        let mut res = HashMap::<MIME, Vec<super::MagicRule>>::new();
        
        for x in tuplevec {
            res.insert(x.0, x.1);
        }
        
        Ok(res)
        
    }

    /// Loads the given magic file and outputs a vector of MagicEntry structs
    #[cfg(feature="sys_fdo_magic")]
    pub fn from_filepath(filepath: &str) -> Result<HashMap<MIME, Vec<super::MagicRule>>, String>{
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

/// Init whatever is needed for test mod to work
pub mod init {

    extern crate std;
    use std::collections::HashMap;
    use MIME;
    #[cfg(feature="sys_fdo_magic")] use std::io::prelude::*;
    #[cfg(feature="sys_fdo_magic")] use std::io::BufReader;
    #[cfg(feature="sys_fdo_magic")] use std::fs::File;

    /// Read all subclass lines from file
    #[cfg(feature="sys_fdo_magic")]
    fn read_subclasses() -> Result<Vec<(MIME, MIME)>, std::io::Error> {
    
        let f = File::open("/usr/share/mime/subclasses")?;
        let r = BufReader::new(f);
        let mut subclasses = Vec::<(MIME, MIME)>::new();
        
        for x in r.lines() {
            let line = x?;
            
            let child = line.split_whitespace().nth(0).unwrap_or("").to_string();
            let parent = line.split_whitespace().nth(1).unwrap_or("").to_string();
            
            subclasses.push( (parent, child) );
        }
        
        Ok(subclasses)
    }
    #[cfg(not(feature="sys_fdo_magic"))]
    fn read_subclasses() -> Result<Vec<(MIME, MIME)>, std::io::Error> {
    
        let r = include_str!("subclasses");
        let mut subclasses = Vec::<(MIME, MIME)>::new();
        
        for line in r.lines() {
            let child = convmime!(line.split_whitespace().nth(0).unwrap_or(""));
            let parent = convmime!(line.split_whitespace().nth(1).unwrap_or(""));
            
            subclasses.push( (parent, child) );
        }
        
        Ok(subclasses)
    }
    
    // Get filetype aliases
    #[cfg(feature="sys_fdo_magic")]
    fn read_aliaslist() -> Result<HashMap<MIME, MIME>, std::io::Error> {
        let faliases = File::open("/usr/share/mime/aliases")?;
        let raliases = BufReader::new(faliases);
        let mut aliaslist = HashMap::<MIME, MIME>::new();
        
        for x in raliases.lines() {
            let line = x?;
        
            let a = convmime!(line.split_whitespace().nth(0).unwrap_or(""));
            let b = convmime!(line.split_whitespace().nth(1).unwrap_or(""));
            aliaslist.insert(a,b);
        }
        
        let aliaslist = aliaslist;
        Ok(aliaslist)
    }
    #[cfg(not(feature="sys_fdo_magic"))]
    fn read_aliaslist() -> Result<HashMap<MIME, MIME>, std::io::Error> {
        let raliases = include_str!("aliases");
        let mut aliaslist = HashMap::<MIME, MIME>::new();
        
        for line in raliases.lines() {
            let a = convmime!(line.split_whitespace().nth(0).unwrap_or(""));
            let b = convmime!(line.split_whitespace().nth(1).unwrap_or(""));
            aliaslist.insert(a,b);
        }
        
        let aliaslist = aliaslist;
        Ok(aliaslist)
    }
    
    /// Get list of supported MIME types
    #[cfg(not(feature="staticmime"))]
    pub fn get_supported() -> Vec<MIME> {
        super::ALLRULES.keys().map(|x| x.clone()).collect()
    }
    #[cfg(feature="staticmime")]
    pub fn get_supported() -> Vec<MIME> {
        super::ALLRULES.keys().map(|x| *x).collect()
    }

    /// Get list of parent -> child subclass links
    pub fn get_subclasses() -> Vec<(MIME, MIME)> {
    
        let mut subclasses = read_subclasses().unwrap_or(Vec::<(MIME, MIME)>::new());
        let aliaslist = read_aliaslist().unwrap_or(HashMap::<MIME, MIME>::new());
        
        // If child or parent refers to an alias, change it to the real type
        for x in 0..subclasses.len(){
            match aliaslist.get(&subclasses[x].0) {
                Some(alias) => {subclasses[x].0 = alias.clone();}
                None => {}
            }
            match aliaslist.get(&subclasses[x].1) {
                Some(alias) => {subclasses[x].1 = alias.clone();}
                None => {}
            }
        }
        
        subclasses
    }
}

// Functions to check if a file matches a magic entry
pub mod test {

    extern crate std;
    
    fn from_u8_singlerule(file: &[u8], rule: super::MagicRule) -> bool {
        
        // Check if we're even in bounds
        let bound_min = std::cmp::min(
            rule.start_off as usize,
            rule.val.len()
        );
        let bound_max =
            std::cmp::min(
            (
                rule.start_off as usize +
                rule.val_len as usize +
                rule.region_len as usize
            ),
            rule.val.len()
        );

        if (file.len()) < bound_max {
            return false;
        }
        
        // Define our testing slice
        let ref x: Vec<u8> = file.iter().take(file.len()).map(|&x| x).collect();
        let testarea: Vec<u8> = x.iter().skip(bound_min).take(bound_max - bound_min).map(|&x| x).collect();
        //println!("{:?}, {:?}, {:?}\n", file, testarea, rule.val);
        
        // Apply mask to value
        
        // Search down until we find a hit
        let mut y = Vec::<u8>::with_capacity(file.len());
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

        false
    }
    
    pub fn can_check(mimetype: &str) -> bool {
        super::ALLRULES.contains_key(mimetype)
    }

    /// Test against all rules
    // This got really complicated really fast...
    pub fn from_u8(file: &[u8], mimetype: &str) -> bool {
    
        // Get magic ruleset
        let magic_rules = match super::ALLRULES.get(mimetype) {
            Some(item) => item,
            None => return false // No rule for this mime
        };
    
        // Test every given rule
        for i in 0..magic_rules.len() {
        
            // If there aren't any rules ahead of us, just test the rule
            if magic_rules.len() - i < 2 {
                let ref x = magic_rules[i];
                match from_u8_singlerule(&file, x.clone()) {
                    true => return true,
                    false => continue,
                };
            
            // If there are rules ahead of us...
            } else {
                let x = magic_rules[i..magic_rules.len()].windows(2).next();
                
                // Make sure that assumption was true
                let y = match x {
                    Some(out) => out,
                    None => {continue;},
                };
                
                // Test the current rule
                match from_u8_singlerule(&file, y[0].clone()) {
                    true => {
                        // Check next indent level if needed
                        if y[1].indent_level >= y[0].indent_level {
                            continue;
                        // Next indent level is lower, so this must be it
                        } else {
                            return true;
                        }
                    },
                    // No match, so keep searching
                    false => continue,
                };
            }
        }
        
        false
    }
    
    pub fn from_filepath(filepath: &str, mimetype: &str) -> Result<bool, std::io::Error>{
        use std::io::prelude::*;
        use std::io::BufReader;
        use std::fs::File;
        
        // Get magic ruleset
        let magic_rules = match super::ALLRULES.get(mimetype) {
            Some(item) => item,
            None => return Ok(false) // No rule for this mime
        };

        // Get # of bytes to read
        let mut scanlen:u64 = 0;
        for x in magic_rules {
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
        let mut b = Vec::<u8>::with_capacity(scanlen as usize);
        r.take(scanlen).read_to_end(&mut b)?;
        
        Ok(from_u8(b.as_slice(), mimetype))
    }

}

