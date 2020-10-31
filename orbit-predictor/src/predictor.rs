pub mod sgp4 {
  pub fn predict(tle: tle_parser::TLE) {
    println!("This is SGP4");
    println!("{:?}", tle);
  }
}
