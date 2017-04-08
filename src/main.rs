extern crate petgraph;
extern crate mime;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::collections::HashMap;
use petgraph::prelude::*;
use petgraph::dot::{Dot, Config};

fn mimelist_init() -> Result<Vec<String>, std::io::Error> {
    let ftypes = File::open("/usr/share/mime/types")?;
    //let ftypes = File::open("/usr/share/mime/subclasses")?;
    let rtypes = BufReader::new(ftypes);
    let mut mimelist = Vec::<String>::new();
    
    for line in rtypes.lines() {
        let mime = line?.split_whitespace().nth(0).unwrap_or("").to_string();
        mimelist.push(mime);
    }
    
    // Don't forget "all/all"!
    /*{
        let mime = "all/all".to_string();
        mimelist.push(mime);
    }*/
    
    let mimelist = mimelist;
    Ok(mimelist)
}

fn graph_init(mimelist: &Vec<String> ) -> Result<DiGraph<String, u32>, std::io::Error> {

    let fsubclasses = File::open("/usr/share/mime/subclasses")?;
    let rsubclasses = BufReader::new(fsubclasses);
    let mut graph = DiGraph::<String, u32>::new();
    let mut added_mimes = HashMap::<String, NodeIndex>::new();
    
    let mut node_text: NodeIndex = NodeIndex::default();
    let mut node_octet: NodeIndex = NodeIndex::default();
    let mut node_allall: NodeIndex = NodeIndex::default();
    let mut node_allfiles: NodeIndex = NodeIndex::default();
    
    // Create all nodes
    for mimetype in mimelist.iter() {
        let node = graph.add_node(mimetype.clone());
        added_mimes.insert(mimetype.clone(), node);
        
        // Record well-used parent types now
        if mimetype == "text/plain" {
            node_text = node;
        } else if mimetype == "application/octet-stream" {
            node_octet = node;
        } else if mimetype == "all/all" {
            node_allall = node;
        } else if mimetype == "all/allfiles" {
            node_allfiles = node;
        }
    }
    
    if node_text == NodeIndex::default() {
        let mimetype = "text/plain".to_string();
        node_text = graph.add_node(mimetype.clone());
        added_mimes.insert(mimetype.clone(), node_text);
    }
    
    if node_octet == NodeIndex::default() {
        let mimetype = "application/octet-stream".to_string();
        node_octet = graph.add_node(mimetype.clone());
        added_mimes.insert(mimetype.clone(), node_octet);
    }
    
    let node_text = node_text;
    let node_octet = node_octet;

    
    // If a relation exists, add child to parent node
    for line in rsubclasses.lines() {
        let line_raw = line?;
        let child_raw = line_raw.split_whitespace().nth(0).unwrap_or("").to_string();
        let parent_raw = line_raw.split_whitespace().nth(1).unwrap_or("").to_string();
        
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
        
        graph.update_edge(parent, child, 1);
    }
    
    
    //Otherwise, add to applicaton/octet-stream, all/all, or text/plain, depending on top-level
    graph.update_edge(node_octet, node_text, 1);
    graph.update_edge(node_allall, node_allfiles, 1);
    graph.update_edge(node_allfiles, node_octet, 1);
    
    let mut edge_list = Vec::<(NodeIndex, NodeIndex)>::new();
    for mimenode in graph.externals(Incoming) {
        
        let ref mimetype = graph[mimenode];
        let toplevel = mimetype.split("/").nth(0).unwrap_or("");
        
        if mimenode == node_text || mimenode == node_octet {
            continue;
        }
        
        if toplevel == "text" {
            edge_list.push( (node_text, mimenode) );
        } else if toplevel == "inode" {
            edge_list.push( (node_allall, mimenode) );
        } else {
            edge_list.push( (node_octet, mimenode) );
        }
    }
    
    graph.extend_with_edges(edge_list);
    
    let graph = graph;
    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

    Ok(graph)
}

fn main() {

    // Create HashMap with all nodes
    let mimelist: Vec<String>;
    match mimelist_init() {
        Err(why) => panic!("{:?}", why),
        Ok(out) => {
            mimelist = out;
        },
    }
    
    //println!("{:?}", type_hashmap);

    match graph_init(&mimelist) {
        Err(why) => panic!("{:?}", why),
        Ok(graph) => {
            let type_graph = graph;
        },
    };
}
