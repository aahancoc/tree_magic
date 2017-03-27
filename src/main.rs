use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

fn main() {
    let f = File::open("/usr/share/mime/types").ok().expect("mime/types file not found. Is this a *nix system?");
    let reader = BufReader::new(f);

    for line in reader.lines() {
        println!("{}", line.ok().expect("Could not read line"));
    }
}
