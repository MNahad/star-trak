extern crate tle_parser;
mod predictor;
use predictor::sgp4;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

fn read_file(path: &str) -> Result<Vec<String>, String> {
  let mut cnt = 0;
  let mut tle_block = String::new();
  let mut tle_array = Vec::new();
  if let Ok(file) = File::open(path) {
    let lines = BufReader::new(file).lines();
    for line_result in lines {
      if let Ok(line) = line_result {
        tle_block.push_str(&line);
        tle_block.push('\n');
        cnt += 1;
        if cnt == 3 {
          tle_array.push(String::from(&tle_block));
          tle_block = String::new();
          cnt = 0;
        }
      }
    }
    Ok(tle_array)
  } else {
    Err(String::from("FILE ERROR"))
  }
}

fn main() {
  if let Ok(raw_tles) = read_file("./src/starlink.txt") {
    for tle in raw_tles {
      let orbit = tle_parser::parse(&tle);
      match orbit {
        Ok(tle) => sgp4::predict(tle),
        Err(_) => println!("ERROR")
      }
    }
  }
}
