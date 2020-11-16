use sgp4::Elements;
use sgp4::Constants;
use chrono::Utc;

pub struct Sat {
  pub name: Option<String>,
  pub position: [f64; 3],
  pub velocity: [f64; 3],
  pub epoch: String,
  pub elapsed: i64,
}

pub fn predict(elements_group: &Vec<Elements>) -> Result<Vec<Sat>, Box<dyn std::error::Error>> {
  let mut predictions: Vec<Sat> = Vec::new();
  for elements in elements_group {
    if let Ok(constants) = Constants::from_elements(elements) {
      let elapsed = Utc::now().naive_utc().signed_duration_since(elements.datetime);
      if let Ok(prediction) = constants.propagate((elapsed.num_minutes()) as f64) {
        predictions.push(Sat {
          name: elements.object_name.clone(),
          position: prediction.position,
          velocity: prediction.velocity,
          epoch: elements.datetime.to_string(),
          elapsed: elapsed.num_minutes(),
        });
      }
    }
  }
  Ok(predictions)
}
