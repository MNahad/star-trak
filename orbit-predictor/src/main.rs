use std::io::{ErrorKind, Error};
mod predictor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let res = ureq::get("https://celestrak.com/NORAD/elements/gp.php")
    .query("GROUP", "starlink")
    .query("FORMAT", "json")
    .call();
  if res.error() {
    Err(Box::new(Error::new(ErrorKind::Other, format!(
      "network error {}: {}",
      res.status(),
      res.into_string()?
    ))))
  } else {
    if let Ok(sats) = predictor::predict(&res.into_json_deserialize()?) {
      for sat in &sats {
        println!("{}", sat.name.as_ref().unwrap_or(&String::from("STARLINK")));
        println!("{:?}", sat.position);
        println!("{:?}", sat.velocity);
        println!("{}", sat.epoch);
        println!("{}\n", sat.elapsed);
      }
    }
    Ok(())
  }
}
