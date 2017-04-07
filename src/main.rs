extern crate petgraph;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::collections::HashMap;
//use std::collections::hash_map;
use std::collections::HashSet;
//use std::hash::Hasher;
use petgraph::prelude::*;
use petgraph::dot::{Dot, Config};

fn hashset_init() -> Result<HashSet<String>, std::io::Error> {
    //let ftypes = File::open("/usr/share/mime/types")?;
    let ftypes = File::open("/usr/share/mime/subclasses")?;
    let rtypes = BufReader::new(ftypes);
    //let mut hashmap = HashMap::<String, u64>::new();
    let mut hashset = HashSet::<String>::new();
    //let mut hasher = hash_map::DefaultHasher::new();
    
    for line in rtypes.lines() {
        //let line_raw = line?;
        let line_raw = line?.split_whitespace().nth(0).unwrap_or("").to_string();
        //hasher.write(line_raw.as_bytes());
        //let hash = hasher.finish();
        
        //hashmap.insert(line_raw, hash);
        hashset.insert(line_raw);
    }
    
    // Don't forget "all/all"!
    {
        let line_raw = "all/all".to_string();
        //hasher.write(line_raw.as_bytes());
        //let hash = hasher.finish();
        
        //hashmap.insert(line_raw, hash);
        hashset.insert(line_raw);
    }
    
    let hashset = hashset;
    Ok(hashset)
}

fn graph_init(allmimes: &HashSet<String> ) -> Result<DiGraph<String, u32>, std::io::Error> {

    let fsubclasses = File::open("/usr/share/mime/subclasses")?;
    let rsubclasses = BufReader::new(fsubclasses);
    //let mut graph = DiGraphMap::<u64, u32>::new();
    let mut graph = DiGraph::<String, u32>::new();
    //let mut found_mimes = HashSet::<String>::new();
    //let mut hasher = hash_map::DefaultHasher::new();
    let mut added_mimes = HashMap::<String, NodeIndex>::new();
    
    // Create all nodes
    for name in allmimes.iter() {
        let node = graph.add_node(name.clone());
        added_mimes.insert(name.clone(), node);
    }

    
    // If a relation exists, add child to parent node
    for line in rsubclasses.lines() {
        let line_raw = line?;
        let child_raw = line_raw.split_whitespace().nth(0).unwrap_or("").to_string();
        let parent_raw = line_raw.split_whitespace().nth(1).unwrap_or("").to_string();
        
        //found_mimes.insert(child_raw.clone());
        
        // Get values of parent and child from HashSet. I hope this works?
        
        //hasher.write(parent_raw.as_bytes());
        //let parent = hasher.finish();
        
        //hasher.write(child_raw.as_bytes());
        //let child = hasher.finish();
        
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
    
    
    // Otherwise, add to applicatom/octet-stream or text/plain, depending on prefix
    //for x in allmimes.difference(&found_mimes) {
    //    println!("{}", x);
    //}
    
    // Okay. How'd we do this in reverse? (That is, given a hash get the string?)
    
    let graph = graph;
    
    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

    Ok(graph)
}

/*fn graph_link() -> Result<Vec<(String, String)>, std::io::Error> {
    let f = File::open("/usr/share/mime/subclasses")?;
    let reader = BufReader::new(f);
    let mut out = Vec::new();

    for line in reader.lines() {
        let line_raw = line?;
        let parent = line_raw.split_whitespace().nth(0).unwrap_or("").to_string();
        let child = line_raw.split_whitespace().nth(1).unwrap_or("").to_string();
        out.push((parent, child));
    }
    
    out.dedup();
    Ok(out)
    
}*/

fn main() {

    // Create HashMap with all nodes
    let type_hashset: HashSet<String>;
    match hashset_init() {
        Err(why) => panic!("{:?}", why),
        Ok(hashset) => {
            type_hashset = hashset;
        },
    }
    
    //println!("{:?}", type_hashmap);

    match graph_init(&type_hashset) {
        Err(why) => panic!("{:?}", why),
        Ok(graph) => {
            let type_graph = graph;
        },
    };
    
    /*match make_type_tree() {
        Err(why) => panic!("{:?}", why),
        Ok(ftypes) => println!("OK!"),
    };*/
}
