use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

fn load_types() -> Result<Vec<String>, std::io::Error> {
    let f = File::open("/usr/share/mime/types")?;
    let reader = BufReader::new(f);
    let mut out = Vec::new();

    for line in reader.lines() {
        out.push(line?);
    }
    
    Ok(out)
    
}

fn load_relations() -> Result<Vec<(String, String)>, std::io::Error> {
    let f = File::open("/usr/share/mime/subclasses")?;
    let reader = BufReader::new(f);
    let mut out = Vec::new();

    for line in reader.lines() {
        let line_raw = line?;
        let parent = line_raw.split_whitespace().nth(0).unwrap_or("").to_string();
        let child = line_raw.split_whitespace().nth(1).unwrap_or("").to_string();
        out.push((parent, child));
    }
    
    Ok(out)
    
}

fn main() {

    match load_relations() {
        Err(why) => panic!("{:?}", why),
        Ok(ftypes) => for ftype in &ftypes {
            println!("{} -> {}", ftype.0, ftype.1);
        },
    };

    /*match load_types() {
        Err(why) => panic!("{:?}", why),
        Ok(ftypes) => for ftype in &ftypes {
            println!("{}", ftype);
        },
    };*/
}
