#[macro_use] extern crate nom;

extern crate petgraph;
extern crate clap;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::collections::HashMap;
use std::collections::HashSet;
use petgraph::prelude::*;
//use petgraph::dot::{Dot, Config};

mod parse;
use parse::*;

// Initialize filetype graph
fn graph_init() -> Result<DiGraph<String, u32>, std::io::Error> {
    
    let mut graph = DiGraph::<String, u32>::new();
    let mut added_mimes = HashMap::<String, NodeIndex>::new();
    
    // Get list of MIME types
    let mut mimelist = magic::init::get_supported();
    mimelist.extend(basetype::init::get_supported());
    mimelist.sort();
    mimelist.dedup();
    let mimelist = mimelist;
    
    // Create all nodes
    for x in mimelist.iter() {
    
        // Do not insert aliases
        let mimetype = x;
        /*match aliaslist.get(x) {
            Some(alias) => {mimetype = alias;}
            None => {mimetype = x;}
        }*/
        
        // Do not insert "x-content/*" or "multipart/*"
        let toplevel = mimetype.split("/").nth(0).unwrap_or("");
        if toplevel == "x-content" || toplevel == "multipart" {
            continue;
        }
    
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
        
        let parent: NodeIndex;
        let child: NodeIndex;
        
        match added_mimes.get(&parent_raw) {
            Some(node) => {parent = *node;}
            None => {continue;}
        }
        
        match added_mimes.get(&child_raw) {
            Some(node) => {child = *node;}
            None => {continue;}
        }
        
        edge_list.insert( (child, parent) );
    }
    
    graph.extend_with_edges(&edge_list);
    
    //Add to applicaton/octet-stream, all/all, or text/plain, depending on top-level
    //(We'll just do it here because having the graph makes it really nice)
    let node_text = match added_mimes.get("text/plain"){
        Some(x) => *x,
        None => graph.add_node("text/plain".to_string())
    };
    let node_octet = match added_mimes.get("application/octet-stream"){
        Some(x) => *x,
        None => graph.add_node("application/octet-stream".to_string())
    };
    let node_allall = match added_mimes.get("all/all"){
        Some(x) => *x,
        None => graph.add_node("all/all".to_string())
    };
    let node_allfiles = match added_mimes.get("all/allfiles"){
        Some(x) => *x,
        None => graph.add_node("all/allfiles".to_string())
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
    //println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

    Ok(graph)
}

/// The meat. Gets the type of a file.
fn get_type_from_filepath(
    node: Option<NodeIndex>,
    typegraph: &DiGraph<String, u32>, 
    magic_ruleset: &HashMap<String, Vec<magic::MagicRule>>,
    filepath: &str
) -> Option<String> {

    // Start at an outside unconnected node if no node given
    let parentnode: NodeIndex;
    
    //println!{">>"};
    
    match node {
        Some(foundnode) => parentnode = foundnode,
        None => {
            match typegraph.externals(Incoming).next() {
                Some(foundnode) => parentnode = foundnode,
                None => panic!("No external nodes found!")
            }
        }
    }
    
    // Walk the children
    let mut children = typegraph.neighbors_directed(parentnode, Outgoing).detach();
    while let Some(childnode) = children.next_node(&typegraph) {
        let ref mimetype = typegraph[childnode];
        
        //println!("{}", mimetype);
        
        let result: Result<bool, std::io::Error>;
        
        // Handle base types
        if basetype::test::can_check(&mimetype){
            result = basetype::test::from_filepath(filepath, &mimetype);
        // Handle via magic
        } else if magic::test::can_check(&mimetype) {

            let rule;
            match magic_ruleset.get(mimetype){
                Some(item) => rule = item,
                None => continue, // ??
            }
            
            result = magic::test::from_filepath(filepath, &mimetype, rule.clone());
        // Nothing can handle this. Somehow.
        } else {
            result = Ok(false);
        }
        
        match result {
            Ok(res) => match res {
                true => {
                    match get_type_from_filepath(
                        Some(childnode), &typegraph, &magic_ruleset, filepath
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
        .about("Finds the MIME type of a file using FD.O Shared MIME database")
        .arg(Arg::with_name("file")
            .required(true)
            .index(1)
            .multiple(true)
        )
        .get_matches();
    let files: Vec<_> = args.values_of("file").unwrap().collect();

    let typegraph: DiGraph<String, u32>;
    match graph_init() {
        Err(why) => panic!("{:?}", why),
        Ok(out) => {
            typegraph = out;
        },
    };
    
    let magic_ruleset: HashMap<String, Vec<magic::MagicRule>>;
    match magic::ruleset::from_filepath("/usr/share/mime/magic") {
        Err(why) => panic!("{:?}", why),
        Ok(out) => {
            magic_ruleset = out;
        },
    }
    
    for x in files {
       println!("{}:\t{:?}", x, get_type_from_filepath(None, &typegraph, &magic_ruleset, x).unwrap_or("inode/none".to_string()));
    }
    
}
