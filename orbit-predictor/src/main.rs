extern crate tle_parser;
mod predictor;
use predictor::sgp4;
use std::fs;

fn main() {
  let data = fs::read_to_string("./src/starlink.txt").expect("Error");
  let orbits = tle_parser::parse(&data);

  match orbits {
    Ok(tle) => sgp4::predict(tle),
    Err(_) => println!("ERROR")
  }
}
