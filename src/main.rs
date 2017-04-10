#[macro_use] extern crate nom;
#[macro_use] extern crate lazy_static;

extern crate petgraph;
extern crate clap;
extern crate tabwriter;

use std::collections::HashMap;
use std::collections::HashSet;
use petgraph::prelude::*;
//use petgraph::dot::{Dot, Config};
use tabwriter::TabWriter;
use std::io::Write;

mod parse;
use parse::*;

pub struct TypeStruct {
    pub graph: DiGraph<String, u32>,
    pub hash: HashMap<String, NodeIndex>
}

lazy_static! {
    static ref TYPE: TypeStruct = {
        graph_init().unwrap_or( TypeStruct{graph: DiGraph::new(), hash: HashMap::new()} )
    };
    
    //static ref TYPEGRAPH: DiGraph<String, u32> = {TYPE.graph};
    //static ref TYPEHASH: HashMap<String, NodeIndex> = {TYPE.1};
}

// Initialize filetype graph
fn graph_init() -> Result<TypeStruct, std::io::Error> {
    
    let mut graph = DiGraph::<String, u32>::new();
    let mut added_mimes = HashMap::<String, NodeIndex>::new();
    
    // Get list of MIME types
    let mut mimelist = magic::init::get_supported();
    mimelist.extend(basetype::init::get_supported());
    mimelist.sort();
    mimelist.dedup();
    let mimelist = mimelist;
    
    // Create all nodes
    for mimetype in mimelist.iter() {
        let node = graph.add_node(mimetype.clone());
        added_mimes.insert(mimetype.clone(), node);
    }
    
    // Get list of edges from each mod's init submod
    // TODO: Can we iterate over a vector of function/module pointers?
    let mut edge_list_raw = basetype::init::get_subclasses();
    edge_list_raw.extend(magic::init::get_subclasses());
        
    let mut edge_list = HashSet::<(NodeIndex, NodeIndex)>::new();
    for x in edge_list_raw {
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
            let node = graph.add_node("text/plain".to_string());
            added_mimes.insert("text/plain".to_string(), node);
            node
        }
    };
    let node_octet = match added_mimes_tmp.get("application/octet-stream"){
        Some(x) => *x,
        None => {
            let node = graph.add_node("application/octet-stream".to_string());
            added_mimes.insert("application/octet-stream".to_string(), node);
            node
        }
    };
    let node_allall = match added_mimes_tmp.get("all/all"){
        Some(x) => *x,
        None => {
            let node = graph.add_node("all/all".to_string());
            added_mimes.insert("all/all".to_string(), node);
            node
        }
    };
    let node_allfiles = match added_mimes_tmp.get("all/allfiles"){
        Some(x) => *x,
        None => {
            let node = graph.add_node("all/allfiles".to_string());
            added_mimes.insert("all/allfiles".to_string(), node);
            node
        }
    };
    
    let mut edge_list_2 = HashSet::<(NodeIndex, NodeIndex)>::new();
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

/// The meat. Gets the type of a file.
fn get_type_from_filepath(
    node: Option<NodeIndex>,
    //typegraph: &DiGraph<String, u32>, 
    //magic_ruleset: &HashMap<String, Vec<magic::MagicRule>>,
    filepath: &str
) -> Option<String> {

    // Start at an outside unconnected node if no node given
    let parentnode: NodeIndex;
    
    match node {
        Some(foundnode) => parentnode = foundnode,
        None => {
            match TYPE.graph.externals(Incoming).next() {
                Some(foundnode) => parentnode = foundnode,
                None => panic!("No external nodes found!")
            }
        }
    }
    
    // Walk the children
    let mut children = TYPE.graph.neighbors_directed(parentnode, Outgoing).detach();
    while let Some(childnode) = children.next_node(&TYPE.graph) {
        let ref mimetype = TYPE.graph[childnode];
        
        let result: Result<bool, std::io::Error>;
        
        // Handle base types
        if basetype::test::can_check(&mimetype){
            result = basetype::test::from_filepath(filepath, &mimetype);
        // Handle via magic
        } else if magic::test::can_check(&mimetype) {
            result = magic::test::from_filepath(filepath, &mimetype);
        // Nothing can handle this. Somehow.
        } else {
            result = Ok(false);
        }
        
        match result {
            Ok(res) => match res {
                true => {
                    match get_type_from_filepath(
                        Some(childnode), filepath
                    ) {
                        Some(foundtype) => return Some(foundtype),
                        None => return Some(mimetype.clone()),
                    }
                }
                false => continue,
            },
            Err(why) => panic!("{:?}", why),
        }
    }
    
    None
}

fn main() {

    use clap::{Arg, App};

    let args = App::new("TreeMagic")
        .version("0.1")
        .about("Determines the MIME type of a file by traversing a filetype tree.")
        .arg(Arg::with_name("file")
            .required(true)
            .index(1)
            .multiple(true)
        )
        .get_matches();
    let files: Vec<_> = args.values_of("file").unwrap().collect();
    
    let mut tw = TabWriter::new(vec![]);
    for x in files {
        write!(&mut tw,
            "{}:\t{:?}\n", x, get_type_from_filepath(
                None, x
            ).unwrap_or(
                "inode/empty".to_string()
            )
        ).unwrap();
    }
    
    tw.flush().unwrap();
    let out = String::from_utf8(tw.into_inner().unwrap()).unwrap_or("".to_string());
    println!("{}", out);
    
}
