use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

fn load_types() -> Result<Vec<String>, std::io::Error> {
    let f = File::open("/usr/share/mime/types")?;
    let reader = BufReader::new(f);
    let mut out = Vec::new();

    for line in reader.lines() {
        //println!("{}", line.ok().expect("Could not read line"));
        out.push(line.ok().expect("Could not read line"));
    }
    
    Ok(out)
    
}

fn main() {
    match load_types() {
        Err(why) => panic!("{:?}", why),
        Ok(ftypes) => for ftype in &ftypes {
            println!("{}", ftype);
        },
    }
}
