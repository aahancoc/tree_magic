//! Read magic file bundled in crate

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
		init::read_aliaslist().unwrap_or(FnvHashMap::default())
	};
}

/// Load magic file before anything else.
lazy_static! {
    static ref ALLRULES: FnvHashMap<MIME, DiGraph<MagicRule, u32>> = {
        super::ruleset::from_u8(include_bytes!("magic")).unwrap_or(FnvHashMap::default())
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
    use MIME;
    
    /// Read all subclass lines from file
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

    // Get filetype aliases (not really public but I need it to be)
    pub fn read_aliaslist() -> Result<FnvHashMap<MIME, MIME>, std::io::Error> {
        let raliases = include_str!("aliases");
        let mut aliaslist = FnvHashMap::<MIME, MIME>::default();
        
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
        super::ALLRULES.keys().cloned().collect()
    }
    #[cfg(feature="staticmime")]
    pub fn get_supported() -> Vec<MIME> {
        super::ALLRULES.keys().map(|x| *x).collect()
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
    
}

pub mod check {
    extern crate std;
    extern crate petgraph;
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
    pub fn from_filepath(filepath: &str, mimetype: &str) -> bool{
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
        match f.read_exact(&mut b) {
            Ok(_) => {},
            Err(_) => return false
        }
        
        from_u8(b.as_slice(), mimetype)
    }
}
