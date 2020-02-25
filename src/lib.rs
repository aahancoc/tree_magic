//! `tree_magic` is a Rust crate that determines the MIME type a given file or byte stream. 
//!
//! # About
//! `tree_magic` is designed to be more efficient and to have less false positives compared
//! to the old approach used by `libmagic`, or old-fashioned file extension comparisons.
//!
//! Instead, this loads all known MIME types into a tree based on subclasses. Then, instead
//! of checking against *every* file type, `tree_magic` will traverse down the tree and
//! only check the files that make sense to check.
//!
//! # Features
//! - Very fast perfomance (~150ns to check one file against one type,
//!   between 5,000ns and 100,000ns to find a MIME type.)
//! - Check if a file *is* a certain type.
//! - Handles aliases (ex: `application/zip` vs `application/x-zip-compressed`)
//! - Uses system [FreeDesktop.org magic files](https://specifications.freedesktop.org/shared-mime-info-spec/shared-mime-info-spec-latest.html) 
//!   on Linux systems, and built-in magic file on Windows and macOS.
//! - Can delegate different file types to different "checkers", reducing false positives
//!   by choosing a different method of attack.
//!
//! # Feature flags
//! `cli`:        Enable building of `tmagic` binary
//!
//! `staticmime`: Change output of all `from_*` functions from `String` to `&'static str`.
//!               Disables ability to load system magic files. Slightly faster.
//! # Example
//! ```rust
//! extern crate tree_magic;
//! 
//! // Load a GIF file
//! let input: &[u8] = include_bytes!("../tests/image/gif");
//!
//! // Find the MIME type of the GIF
//! let result = tree_magic::from_u8(input);
//! assert_eq!(result, "image/gif");
//!
//! // Check if the MIME and the file are a match
//! let result = tree_magic::match_u8("image/gif", input);
//! assert_eq!(result, true);
//! ```

#![allow(unused_doc_comments)]
#![allow(dead_code)]
#[macro_use] extern crate nom;
#[macro_use] extern crate lazy_static;
extern crate petgraph;
extern crate fnv;
extern crate parking_lot;

use petgraph::prelude::*;
use fnv::FnvHashMap;
use fnv::FnvHashSet;
//use petgraph::dot::{Dot, Config};
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use parking_lot::RwLock;
use std::sync::Arc;

mod fdo_magic;
mod basetype;

#[cfg(feature="staticmime")] type MIME = &'static str;
#[cfg(not(feature="staticmime"))] type MIME = String;

/// Check these types first
const TYPEORDER: [&'static str; 6] =
[
	"image/png",
	"image/jpeg",
	"image/gif",
	"application/zip",
	"application/x-msdos-executable",
	"application/pdf"
];

struct CheckerStruct {
    from_u8: fn(&[u8], &str, CacheItem) -> bool,
    from_filepath: fn(&Path, &str, CacheItem) -> bool,
    get_supported: fn() -> Vec<MIME>,
    get_subclasses: fn() -> Vec<(MIME, MIME)>,
    get_aliaslist: fn() -> FnvHashMap<MIME, MIME>
}

/// Maximum number of checkers supported with build config.
/// TODO: Find any better way to do this!
#[cfg(not(feature="staticmime"))]
const CHECKERCOUNT: usize = 3;
#[cfg(feature="staticmime")]
const CHECKERCOUNT: usize = 2;

/// List of checker functions
const CHECKERS: [CheckerStruct; CHECKERCOUNT] = 
[
    // Disable sys checker when using staticmime
    #[cfg(not(feature="staticmime"))] CheckerStruct{
        from_u8: fdo_magic::sys::check::from_u8,
        from_filepath: fdo_magic::sys::check::from_filepath,
        get_supported: fdo_magic::sys::init::get_supported,
        get_subclasses: fdo_magic::sys::init::get_subclasses,
        get_aliaslist: fdo_magic::sys::init::get_aliaslist
    },
    CheckerStruct{
        from_u8: fdo_magic::builtin::check::from_u8,
        from_filepath: fdo_magic::builtin::check::from_filepath,
        get_supported: fdo_magic::builtin::init::get_supported,
        get_subclasses: fdo_magic::builtin::init::get_subclasses,
        get_aliaslist: fdo_magic::builtin::init::get_aliaslist
    },
    CheckerStruct{
        from_u8: basetype::check::from_u8,
        from_filepath: basetype::check::from_filepath,
        get_supported: basetype::init::get_supported,
        get_subclasses: basetype::init::get_subclasses,
        get_aliaslist: basetype::init::get_aliaslist
    }
];

/// Mappings between modules and supported mimes (by index in table above)
lazy_static! {
    static ref CHECKER_SUPPORT: FnvHashMap<MIME, usize> = {
        let mut out = FnvHashMap::<MIME, usize>::default();
        for i in 0..CHECKERS.len() {
            for j in (CHECKERS[i].get_supported)() {
                out.insert(j, i);
            }
        }
        out
    };
}

lazy_static! {
    static ref ALIASES: FnvHashMap<MIME, MIME> = {
        let mut out = FnvHashMap::<MIME, MIME>::default();
        for i in 0..CHECKERS.len() {
            out.extend((CHECKERS[i].get_aliaslist)());
        }
        out
    };
}

/// Cache used for each checker for each file
#[derive(Clone)]
pub enum Cache {
     #[cfg(not(feature="staticmime"))] FdoMagicSys(fdo_magic::sys::Cache),
     FdoMagicBuiltin(fdo_magic::builtin::Cache),
     Basetype(basetype::Cache)
}
type CacheItem = Arc<RwLock<Option<Cache>>>;
type CacheContainer = Vec<CacheItem>; // Max number of supported checkers

// I'd love to do this, but it needs unstable rust
/*struct CacheStruct {
    #[cfg(not(feature="staticmime"))] fdo_magic_sys: Option<fdo_magic::sys::Cache>,
    fdo_magic_builtin: Option<fdo_magic::builtin::Cache>,
    basetype: Option<basetype::Cache>
}
impl CacheStruct {
    pub fn new() {
        CacheStruct {
            #[cfg(not(feature="staticmime"))] fdo_magic_sys: None,
            fdo_magic_builtin: None,
            basetype: None
        }
    }
}*/

/// Information about currently loaded MIME types
///
/// The `graph` contains subclass relations between all given mimes.
/// (EX: `application/json` -> `text/plain` -> `application/octet-stream`)
/// This is a `petgraph` DiGraph, so you can walk the tree if needed.
/// 
/// The `hash` is a mapping between MIME types and nodes on the graph.
/// The root of the graph is "all/all", so start traversing there unless
/// you need to jump to a particular node.
pub struct TypeStruct {
    pub graph: DiGraph<MIME, u32>,
    pub hash: FnvHashMap<MIME, NodeIndex>
}

lazy_static! {
    /// The TypeStruct autogenerated at library init, and used by the library.
    pub static ref TYPE: TypeStruct = {
        graph_init().unwrap_or( 
            TypeStruct{
                graph: DiGraph::new(),
                hash: FnvHashMap::default()
            } )
    };
}

/// Convert a &str to a MIME
#[cfg(not(feature="staticmime"))]
macro_rules! convmime {
    ($x:expr) => {$x.to_string()}
}
#[cfg(feature="staticmime")]
macro_rules! convmime {
    ($x:expr) => {$x}
}

/// Convert a MIME to a &str
#[cfg(not(feature="staticmime"))]
macro_rules! unconvmime {
    ($x:expr) => {$x.as_str()}
}
#[cfg(feature="staticmime")]
macro_rules! unconvmime {
    ($x:expr) => {$x}
}

/// Clone a MIME
#[cfg(not(feature="staticmime"))]
macro_rules! clonemime {
    ($x:expr) => {$x.clone()}
}
#[cfg(feature="staticmime")]
macro_rules! clonemime {
    ($x:expr) => {$x}
}

// Initialize filetype graph
fn graph_init() -> Result<TypeStruct, std::io::Error> {
    
    let mut graph = DiGraph::<MIME, u32>::new();
    let mut added_mimes = FnvHashMap::<MIME, NodeIndex>::default();
    
    // Get list of MIME types and MIME relations
    let mut mimelist = Vec::<MIME>::new();
    let mut edgelist_raw = Vec::<(MIME, MIME)>::new();
    for i in 0..CHECKERS.len() {
        mimelist.extend((CHECKERS[i].get_supported)());
        edgelist_raw.extend((CHECKERS[i].get_subclasses)());
    }
    mimelist.sort();
    mimelist.dedup();
    let mimelist = mimelist;
    
    // Create all nodes
    for mimetype in mimelist.iter() {
        let node = graph.add_node(clonemime!(mimetype));
        added_mimes.insert(clonemime!(mimetype), node);
    }
        
    let mut edge_list = FnvHashSet::<(NodeIndex, NodeIndex)>::with_capacity_and_hasher(
        edgelist_raw.len(), Default::default()
    );
    for x in edgelist_raw {
        let child_raw = x.0;
        let parent_raw = x.1;
        
        let parent = match added_mimes.get(&parent_raw) {
            Some(node) => *node,
            None => {continue;}
        };
        
        let child = match added_mimes.get(&child_raw) {
            Some(node) => *node,
            None => {continue;}
        };
        
        edge_list.insert( (child, parent) );
    }
    
    graph.extend_with_edges(&edge_list);
    
    //Add to applicaton/octet-stream, all/all, or text/plain, depending on top-level
    //(We'll just do it here because having the graph makes it really nice)
    let added_mimes_tmp = added_mimes.clone();
    let node_text = match added_mimes_tmp.get("text/plain"){
        Some(x) => *x,
        None => {
            let node = graph.add_node(convmime!("text/plain"));
            added_mimes.insert(convmime!("text/plain"), node);
            node
        }
    };
    let node_octet = match added_mimes_tmp.get("application/octet-stream"){
        Some(x) => *x,
        None => {
            let node = graph.add_node(convmime!("application/octet-stream"));
            added_mimes.insert(convmime!("application/octet-stream"), node);
            node
        }
    };
    let node_allall = match added_mimes_tmp.get("all/all"){
        Some(x) => *x,
        None => {
            let node = graph.add_node(convmime!("all/all"));
            added_mimes.insert(convmime!("all/all"), node);
            node
        }
    };
    let node_allfiles = match added_mimes_tmp.get("all/allfiles"){
        Some(x) => *x,
        None => {
            let node = graph.add_node(convmime!("all/allfiles"));
            added_mimes.insert(convmime!("all/allfiles"), node);
            node
        }
    };
    
    let mut edge_list_2 = FnvHashSet::<(NodeIndex, NodeIndex)>::default();
    for mimenode in graph.externals(Incoming) {
        
        let ref mimetype = graph[mimenode];
        let toplevel = mimetype.split("/").nth(0).unwrap_or("");
        
        if mimenode == node_text || mimenode == node_octet || 
           mimenode == node_allfiles || mimenode == node_allall 
        {
            continue;
        }
        
        if toplevel == "text" {
            edge_list_2.insert( (node_text, mimenode) );
        } else if toplevel == "inode" {
            edge_list_2.insert( (node_allall, mimenode) );
        } else {
            edge_list_2.insert( (node_octet, mimenode) );
        }
    }
    // Don't add duplicate entries
    graph.extend_with_edges(edge_list_2.difference(&edge_list));
    
    let graph = graph;
    let added_mimes = added_mimes;
    //println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

    Ok( TypeStruct{graph: graph, hash: added_mimes} )
}

/// Just the part of from_*_node that walks the graph
fn typegraph_walker<T: Clone>(
    parentnode: NodeIndex,
    input: T,
    cache: &CacheContainer,
    matchfn: fn(&str, T, &CacheContainer) -> bool
) -> Option<MIME> {

    // Pull most common types towards top
    let mut children: Vec<NodeIndex> = TYPE.graph
        .neighbors_directed(parentnode, Outgoing)
        .collect();
        
    for i in 0..children.len() {
        let x = children[i];
        if TYPEORDER.contains(&&*TYPE.graph[x]) {
            children.remove(i);
            children.insert(0, x);
        }
    }

    // Walk graph
    for childnode in children {
        let ref mimetype = TYPE.graph[childnode];
        
        let result = (matchfn)(mimetype, input.clone(), cache);
        match result {
            true => {
                match typegraph_walker(
                    childnode, input, cache, matchfn
                ) {
                    Some(foundtype) => return Some(foundtype),
                    None => return Some(clonemime!(mimetype)),
                }
            }
            false => continue,
        }
    }
    
    None
}

/// Transforms an alias into it's real type
#[cfg(feature="staticmime")]
fn get_alias(mimetype: &str) -> &str {
    match ALIASES.get(mimetype) {
        Some(x) => x,
        None => mimetype
    }
}
#[cfg(not(feature="staticmime"))]
fn get_alias(mimetype: &String) -> &String {
    match ALIASES.get(mimetype) {
        Some(x) => x,
        None => mimetype
    }
}

/// Internal function. Checks if an alias exists, and if it does,
/// then runs match_u8.
fn match_u8_noalias(mimetype: &str, bytes: &[u8], cache: &CacheContainer) -> bool
{
    match CHECKER_SUPPORT.get(mimetype) {
        None => {false},
        Some(y) => (CHECKERS[*y].from_u8)(bytes, mimetype, cache[*y].clone())
    }
}

/// Checks if the given bytestream matches the given MIME type.
///
/// Returns true or false if it matches or not. If the given MIME type is not known,
/// the function will always return false.
/// If mimetype is an alias of a known MIME, the file will be checked agains that MIME.
///
/// # Examples
/// ```rust
/// // Load a GIF file
/// let input: &[u8] = include_bytes!("../tests/image/gif");
///
/// // Check if the MIME and the file are a match
/// let result = tree_magic::match_u8("image/gif", input);
/// assert_eq!(result, true);
/// ```
pub fn match_u8(mimetype: &str, bytes: &[u8]) -> bool
{
    // Transform alias if needed
    let oldmime = convmime!(mimetype);
    let x = unconvmime!(get_alias(&oldmime));
    
    match_u8_noalias(x, bytes, &vec![CacheItem::default(); CHECKERCOUNT])
}


/// Gets the type of a file from a raw bytestream, starting at a certain node
/// in the type graph.
///
/// Returns MIME as string wrapped in Some if a type matches, or
/// None if no match is found under the given node.
/// Retreive the node from the `TYPE.hash` HashMap, using the MIME as the key.
///
/// # Panics
/// Will panic if the given node is not found in the graph.
/// As the graph is immutable, this should not happen if the node index comes from
/// TYPE.hash.
///
/// # Examples
/// ```rust
/// /// In this example, we know we have a ZIP, but we want to see if it's something
/// /// like an Office document that subclasses a ZIP. If it is not, like this example,
/// /// it will return None.
///
/// // Load a ZIP file
/// let input: &[u8] = include_bytes!("../tests/application/zip");
/// 
/// // Get the graph node for ZIP
/// let zipnode = tree_magic::TYPE.hash.get("application/zip").unwrap();
///
/// // Find the MIME type of the ZIP, starting from ZIP.
/// let result = tree_magic::from_u8_node(*zipnode, input);
/// assert_eq!(result, None);
/// ```
pub fn from_u8_node(parentnode: NodeIndex, bytes: &[u8]) -> Option<MIME>
{
	typegraph_walker(parentnode, bytes, &vec![CacheItem::default(); CHECKERCOUNT], match_u8_noalias)
}

/// Gets the type of a file from a byte stream.
///
/// Returns MIME as string.
///
/// # Examples
/// ```rust
/// // Load a GIF file
/// let input: &[u8] = include_bytes!("../tests/image/gif");
///
/// // Find the MIME type of the GIF
/// let result = tree_magic::from_u8(input);
/// assert_eq!(result, "image/gif");
/// ```
pub fn from_u8(bytes: &[u8]) -> MIME
{
    let node = match TYPE.graph.externals(Incoming).next() {
        Some(foundnode) => foundnode,
        None => panic!("No filetype definitions are loaded.")
    };
    from_u8_node(node, bytes).unwrap()
}

/// Internal function. Checks if an alias exists, and if it does,
/// then runs `match_u8`.
fn match_filepath_noalias(mimetype: &str, filepath: &Path, cache: &CacheContainer) -> bool
{
    match CHECKER_SUPPORT.get(mimetype) {
        None => {false},
        Some(y) => (CHECKERS[*y].from_filepath)(filepath, mimetype, cache[*y].clone())
    }
}

/// Check if the given filepath matches the given MIME type.
///
/// Returns true or false if it matches or not, or an Error if the file could
/// not be read. If the given MIME type is not known, it will always return false.
///
/// # Examples
/// ```rust
/// use std::path::Path;
///
/// // Get path to a GIF file
/// let path: &Path = Path::new("tests/image/gif");
///
/// // Check if the MIME and the file are a match
/// let result = tree_magic::match_filepath("image/gif", path);
/// assert_eq!(result, true);
/// ```
pub fn match_filepath(mimetype: &str, filepath: &Path) -> bool 
{
    // Transform alias if needed
    let oldmime = convmime!(mimetype);
    let x = unconvmime!(get_alias(&oldmime));
   
    match_filepath_noalias(x, filepath, &vec![CacheItem::default(); CHECKERCOUNT])
}


/// Gets the type of a file from a filepath, starting at a certain node
/// in the type graph.
///
/// Returns MIME as string wrapped in Some if a type matches, or
/// None if the file is not found or cannot be opened.
/// Retreive the node from the `TYPE.hash` FnvHashMap, using the MIME as the key.
///
/// # Panics
/// Will panic if the given node is not found in the graph.
/// As the graph is immutable, this should not happen if the node index comes from
/// `TYPE.hash`.
///
/// # Examples
/// ```rust
/// /// In this example, we know we have a ZIP, but we want to see if it's something
/// /// like an Office document that subclasses a ZIP. If it is not, like this example,
/// /// it will return None.
/// use std::path::Path;
///
/// // Get path to a ZIP file
/// let path: &Path = Path::new("tests/application/zip");
/// 
/// // Get the graph node for ZIP
/// let zipnode = tree_magic::TYPE.hash.get("application/zip").unwrap();
///
/// // Find the MIME type of the ZIP, starting from ZIP.
/// let result = tree_magic::from_filepath_node(*zipnode, path);
/// assert_eq!(result, None);
/// ```
pub fn from_filepath_node(parentnode: NodeIndex, filepath: &Path) -> Option<MIME> 
{
    // We're actually just going to thunk this down to a u8
    // unless we're checking via basetype for speed reasons.
    
    // Ensure it's at least a application/octet-stream
    if !match_filepath("application/octet-stream", filepath){
        // Check the other base types
        return typegraph_walker(parentnode, filepath, &vec![CacheItem::default(); CHECKERCOUNT], match_filepath_noalias);
    }
    
    // Load the first 2K of file and parse as u8
    // for batch processing like this
    //
    // TODO: Use cache to only get what we need to when we need to
    // and then change code so that we keep calling this function
    // when walking tree.
    let f = match File::open(filepath) {
        Ok(x) => x,
        Err(_) => return None // How?
    };
    let r = BufReader::new(f);
    let mut b = Vec::<u8>::new();
    match r.take(2048).read_to_end(&mut b) {
        Ok(_) => {},
        Err(_) => return None // Also how?
    }
    
    from_u8_node(parentnode, b.as_slice())
}

/// Gets the type of a file from a filepath.
///
/// Does not look at file name or extension, just the contents.
/// Returns MIME as string.
///
/// # Examples
/// ```rust
/// use std::path::Path;
///
/// // Get path to a GIF file
/// let path: &Path = Path::new("tests/image/gif");
///
/// // Find the MIME type of the GIF
/// let result = tree_magic::from_filepath(path);
/// assert_eq!(result, "image/gif");
/// ```
pub fn from_filepath(filepath: &Path) -> MIME {

    let node = match TYPE.graph.externals(Incoming).next() {
        Some(foundnode) => foundnode,
        None => panic!("No filetype definitions are loaded.")
    };
    
    from_filepath_node(node, filepath).unwrap()
}

/// Determines if a MIME is an alias of another MIME
///
/// If this returns true, that means the two MIME types are equivalent.
/// If this returns false, either one of the MIME types are missing, or they are different.
/// If you're using the `staticmime` feature flag, input is a &'static str.
/// Otherwise it is a String.
///
/// # Examples
/// ```
/// let mime1 = "application/zip".to_string();
/// let mime2 = "application/x-zip-compressed".to_string();
///
/// assert_eq!( tree_magic::is_alias(mime1, mime2), true );
pub fn is_alias(mime1: MIME, mime2: MIME) -> bool {
    let x = get_alias(&mime1);
    let y = get_alias(&mime2);
    
    #[cfg(feature="staticmime")]
    return x == mime2 || y == mime1;
    #[cfg(not(feature="staticmime"))]
    return *x == mime2 || *y == mime1;
}
