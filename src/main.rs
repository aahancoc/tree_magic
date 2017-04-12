//extern crate tree_magic;
extern crate clap;
extern crate tabwriter;
extern crate tree_magic;

use tabwriter::TabWriter;
use std::io::prelude::*;

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
    for x in files {
        write!(&mut tw,
            "{}:\t{:?}\n", x, tree_magic::from_filepath(x).unwrap_or(convmime!("inode/empty"))
        ).unwrap();
    }
    
    tw.flush().unwrap();
    let out = String::from_utf8(tw.into_inner().unwrap()).unwrap_or("".to_string());
    println!("{}", out);
    
}
