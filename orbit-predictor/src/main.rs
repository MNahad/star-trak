extern crate tle_parser;

use tle_parser::parse;
use std::fs;

fn main() {
    let data = fs::read_to_string("./src/starlink.txt").expect("Error");
    let orbits = parse(&data);

    match orbits {
        Ok(t) => println!("{:?}", t),
        Err(_) => println!("ERROR")
    }
}
