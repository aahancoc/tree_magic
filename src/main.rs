//extern crate tree_magic;
extern crate clap;
extern crate tabwriter;
extern crate crossbeam;
extern crate tree_magic;

use tabwriter::TabWriter;
use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[cfg(not(feature="staticmime"))]
macro_rules! convmime {
    ($x:expr) => {$x.to_string()}
}
#[cfg(feature="staticmime")]
macro_rules! convmime {
    ($x:expr) => {$x}
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
    let (tx, rx) = mpsc::channel();
    //let list = Arc::new(Mutex::new(Vec::<String>::new()));
    
    crossbeam::scope(|scope| {
        for x in files {
            let tx = tx.clone();
            //let list = list.clone();
            scope.spawn(move || {
                let result = tree_magic::from_filepath(x).unwrap_or("inode/empty");
                let result = format!("{}:\t{:?}\n", x, result);
                //let ref mut list = *list.lock().unwrap();
                //list.push(result);
                tx.send(result).unwrap();
            });
        }
    });
    drop(tx);
    
    let mut list: Vec<String> = rx.iter().collect();
    //let ref mut list = *list.lock().unwrap();
    list.sort();
    for x in list {
        write!(&mut tw, "{}", x).unwrap();
    }
    
    tw.flush().unwrap();
    let out = String::from_utf8(tw.into_inner().unwrap()).unwrap_or("".to_string());
    println!("{}", out);
    
}
