//extern crate tree_magic;
extern crate clap;
extern crate tabwriter;
extern crate tree_magic;
extern crate walkdir;
extern crate scoped_threadpool;
extern crate num_cpus;

use tabwriter::TabWriter;
use std::io::prelude::*;
use std::sync::mpsc;
use walkdir::{WalkDir};
use scoped_threadpool::Pool;

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
        .arg(Arg::with_name("recursive")
            .short("r")
            .help("Search directories recursively")
        )
        .arg(Arg::with_name("match")
            .short("m")
            .long("match")
            .use_delimiter(true)
            .takes_value(true)
            .help("Print only files that match given MIME")
        )
        .arg(Arg::with_name("ugly")
            .long("ugly")
            .help("Print results as they come in, at expense of tab alignment")
        )
        .get_matches();
    let mut files: Vec<String> = args.values_of("file")
        .unwrap()
        .map(|x| x.to_string())
        .collect();
    
    let mut tw = TabWriter::new(vec![]);
    let (tx, rx) = mpsc::channel();
    
    // Get recursive results if needed
    if args.is_present("recursive") {
        //println!("Recursive!");
        for dir in files.clone() {
            let entries = WalkDir::new(dir).into_iter().filter_map(|e| e.ok());
            for entry in entries {
                files.push(entry.path()
                    .to_str()
                    .unwrap()
                    .to_string()
                );
            }
        }
        //println!("{:#?}", files);
    }
    
    let mut pool = Pool::new(num_cpus::get() as u32);
    let is_ugly = args.is_present("ugly");
    
    pool.scoped(|scope| {
        for x in files {
            let tx = tx.clone();
            scope.execute(move || {
                let result = tree_magic::from_filepath(x.as_str());
                let result = result.unwrap_or(convmime!("inode/none"));
                let result = format!("{}:\t{:?}", x, result);
                if is_ugly {
                    println!("{}", result);
                } else {
                    tx.send(result + "\n").unwrap_or_default();
                }
            });
        }
    });
    drop(tx);
    
    if !is_ugly {
        let mut list: Vec<_> = rx.iter().collect();
        list.sort();
        list.dedup();
        for x in list {
            write!(&mut tw, "{}", x).unwrap();
        }
        
        tw.flush().unwrap();
        let out = String::from_utf8(tw.into_inner().unwrap()).unwrap_or("".to_string());
        println!("{}", out);
    }
    
}
