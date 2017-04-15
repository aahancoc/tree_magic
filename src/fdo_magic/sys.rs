//! Read magic file(s) on user's computer
//!
//! May or may not work, depending on platform, but in that case
//! this will just say it can't handle any file types, and will
//! never be invoked.

extern crate std;
extern crate petgraph;
extern crate fnv;
use petgraph::prelude::*;
use fnv::FnvHashMap;
use MIME;
use super::MagicRule;

/// Preload alias list
lazy_static! {
	static ref ALIASES: FnvHashMap<MIME, MIME> = {
		init::get_aliaslist()
	};
}

/// Load magic file before anything else.
/// sys_fdo_magic always disabled on Windows.
lazy_static! {
    static ref ALLRULES: FnvHashMap<MIME, DiGraph<MagicRule, u32>> = {
        super::ruleset::from_filepath("/usr/share/mime/magic").unwrap_or(FnvHashMap::default())
    };
}

#[cfg(not(feature="staticmime"))]
macro_rules! convmime {
    ($x:expr) => {$x.to_string()}
}
#[cfg(feature="staticmime")]
macro_rules! convmime {
    ($x:expr) => {$x}
}

pub mod init {
    extern crate std;
    extern crate fnv;
    use fnv::FnvHashMap;
    use std::io::prelude::*;
    use std::io::BufReader;
    use std::fs::File;
    use MIME;
    
    /// Read all subclass lines from file
    fn read_subclasses() -> Result<Vec<(MIME, MIME)>, std::io::Error> {
    
        let f = File::open("/usr/share/mime/subclasses")?;
        let r = BufReader::new(f);
        let mut subclasses = Vec::<(MIME, MIME)>::new();
        
        for x in r.lines() {
            let line = x?;
            
            let child = convmime!(line.split_whitespace().nth(0).unwrap_or(""));
            let parent = convmime!(line.split_whitespace().nth(1).unwrap_or(""));
            
            subclasses.push( (parent, child) );
        }
        
        Ok(subclasses)
    }
    
    // Get filetype aliases
    fn read_aliaslist() -> Result<FnvHashMap<MIME, MIME>, std::io::Error> {
        let faliases = File::open("/usr/share/mime/aFnvHashMap::default()liases")?;
        let raliases = BufReader::new(faliases);
        let mut aliaslist = FnvHashMap::<MIME, MIME>::default();
        
        for x in raliases.lines() {
            let line = x?;
        
            let a = convmime!(line.split_whitespace().nth(0).unwrap_or(""));
            let b = convmime!(line.split_whitespace().nth(1).unwrap_or(""));
            aliaslist.insert(a,b);
        }
        
        let aliaslist = aliaslist;
        Ok(aliaslist)
    }
    
    pub fn get_aliaslist() -> FnvHashMap<MIME, MIME> {
        read_aliaslist().unwrap_or(FnvHashMap::default())
    }
    
    /// Get list of parent -> child subclass links
    #[cfg(not(feature="staticmime"))]
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
    /// Return empty list if using staticmime
    #[cfg(feature="staticmime")]
    pub fn get_subclasses() -> Vec<(MIME, MIME)> {
        Vec::<(MIME, MIME)>::new()
    }
    
    /// Get list of supported MIME types
    #[cfg(not(feature="staticmime"))]
    pub fn get_supported() -> Vec<MIME> {
        super::ALLRULES.keys().map(|x| convmime!(x)).collect()
    }
    
    /// Return empty list if using staticmime
    #[cfg(feature="staticmime")]
    pub fn get_supported() -> Vec<MIME> {
        Vec::<MIME>::new()
    }
}

pub mod check {
    extern crate std;
    extern crate petgraph;
    use std::path::Path;
    use petgraph::prelude::*;
    use fdo_magic;

    /// Test against all rules
    pub fn from_u8(file: &[u8], mimetype: &str) -> bool {
		
		// Get mimetype in case user provides alias
		let mimetype = match super::ALIASES.get(mimetype) {
			None => mimetype,
			Some(x) => x
		};
    
        // Get magic ruleset
        let graph = match super::ALLRULES.get(mimetype) {
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
    pub fn from_filepath(filepath: &Path, mimetype: &str) -> bool{
        use std::fs::File;
        use std::io::Read;
        
        // Get magic ruleset
        let magic_rules = match super::ALLRULES.get(mimetype) {
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
        
        let mut f = match File::open(filepath) {
            Ok(x) => x,
            Err(_) => return false
        };
        let mut b = Vec::<u8>::with_capacity(scanlen);
        
        // Fill up vector with something
        for i in 0..scanlen {
            let _ = i;
            b.push(0);
        }
//         
        match f.read_exact(&mut b) {
            Ok(_) => {},
            Err(_) => return false
        }
        
        from_u8(b.as_slice(), mimetype)
    }
}
