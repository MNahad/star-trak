use std::f64::consts::PI;

pub struct Satellite<'a> {
  timing: Timing,
  coords: Coords,
  elements: &'a sgp4::Elements,
}

impl Satellite<'_> {
  pub fn get_norad_id(&self) -> &u64 {
    &self.elements.norad_id
  }
  pub fn get_name(&self) -> &String {
    self.elements.object_name.as_ref().unwrap()
  }
  pub fn get_international_designator(&self) -> &String {
    self.elements.international_designator.as_ref().unwrap()
  }
  pub fn get_geodetic_position(&self) -> &Geodetic {
    &self.coords.position_geodetic
  }
  pub fn get_timestamp_s(&self) -> &i64 {
    &self.timing.timestamp_s
  }
}

pub struct Geodetic {
  pub lat_deg: f64,
  pub lon_deg: f64,
  pub alt_km: f64,
}

struct Cartesian {
  x: f64,
  y: f64,
  z: f64,
}

struct Coords {
  position_geodetic: Geodetic,
  position_eci_km: Cartesian,
  velocity_eci_km_s: Cartesian,
}

struct Timing {
  timestamp_s: i64,
  elapsed_since_epoch_ms: i64,
}

pub fn propagate(
  elements_group: &Vec<sgp4::Elements>,
) -> Result<Vec<Satellite>, Box<dyn std::error::Error>> {
  let mut predictions: Vec<Satellite> = Vec::new();
  for elements in elements_group {
    if let Ok(constants) = sgp4::Constants::from_elements(elements) {
      let timestamp = chrono::Utc::now().naive_utc();
      let elapsed_ms = timestamp.signed_duration_since(elements.datetime).num_milliseconds();
      if let Ok(prediction) = constants.propagate((elapsed_ms as f64) / (1000.0 * 60.0)) {
        let position_eci_km = Cartesian {
          x: prediction.position[0],
          y: prediction.position[1],
          z: prediction.position[2],
        };
        let velocity_eci_km_s = Cartesian {
          x: prediction.velocity[0],
          y: prediction.velocity[1],
          z: prediction.velocity[2],
        };
        let position_geodetic = eci_to_geodetic(
          &position_eci_km,
          &elements.epoch(),
          &((elapsed_ms as f64) / 1000.0),
        );
        predictions.push(Satellite {
          coords: Coords {
            position_eci_km,
            velocity_eci_km_s,
            position_geodetic,
          },
          timing: Timing {
            timestamp_s: timestamp.timestamp(),
            elapsed_since_epoch_ms: elapsed_ms,
          },
          elements,
        });
      }
    }
  }
  Ok(predictions)
}

fn eci_to_geodetic(
  position_eci_km: &Cartesian,
  epoch_y: &f64,
  elapsed_since_epoch_s: &f64,
) -> Geodetic {
  let omega_e_rad_s = 7.29211510e-5;
  let theta_g = (
    sgp4::iau_epoch_to_sidereal_time(*epoch_y) + omega_e_rad_s * *elapsed_since_epoch_s
  ).rem_euclid(2.0 * PI);
  let theta = position_eci_km.y.atan2(position_eci_km.x);
  let theta = if theta < 0.0 { theta + 2.0 * PI } else { theta };
  let delta_theta = theta - theta_g;
  let delta_theta = if delta_theta > PI { delta_theta - 2.0 * PI } else { delta_theta };
  let delta_theta = if delta_theta < -PI { delta_theta + 2.0 * PI } else { delta_theta };
  let lon_deg = delta_theta * 180.0 / PI;
  let r_km = (position_eci_km.x.powi(2) + position_eci_km.y.powi(2)).sqrt();
  let (lat_deg, alt_km) = compute_geodetic_values(&r_km, &position_eci_km.z);
  Geodetic {
    lat_deg,
    lon_deg,
    alt_km,
  }
}

fn compute_geodetic_values(r_km: &f64, z_km: &f64) -> (f64, f64) {
  // Refer to
  // J. Zhu, "Conversion of Earth-centered Earth-fixed coordinates to geodetic coordinates,"
  // IEEE Transactions on Aerospace and Electronic Systems, vol 30, pp 957-961, 1994.
  let a_sq = 6378137.0_f64.powi(2);
  let b_sq = 6356752.3142_f64.powi(2);
  let e_sq = 0.00669437999014_f64;
  let e_two_sq = 0.00673949674228_f64;
  let r = *r_km * 1000.0;
  let r_sq = r.powi(2);
  let z = *z_km * 1000.0;
  let z_sq = z.powi(2);
  let ee_sq = a_sq - b_sq;
  let ff = 54.0 * b_sq * z_sq;
  let gg = r_sq + ((1.0 - e_sq) * z_sq) - (e_sq * ee_sq);
  let cc = (e_sq.powi(2) * ff * r_sq) * gg.powi(3).recip();
  let ss = (1.0 + cc + (cc.powi(2) + 2.0 * cc).sqrt()).cbrt();
  let pp = ff * (3.0 * (ss + ss.recip() + 1.0).powi(2) * gg.powi(2)).recip();
  let qq = (1.0 + 2.0 * e_sq.powi(2) * pp).sqrt();
  let r_o = ((-(pp * e_sq * r)) * (1.0 + qq).recip()) +
    (
      (0.5 * a_sq * (1.0 + qq.recip()))
      - ((pp * (1.0 - e_sq) * z_sq) * (qq * (1.0 + qq)).recip())
      - (0.5 * pp * r_sq)
    ).sqrt();
  let uu = ((r - e_sq * r_o).powi(2) + z_sq).sqrt();
  let vv = ((r - e_sq * r_o).powi(2) + (1.0 - e_sq) * z_sq).sqrt();
  let z_o = b_sq * z * (a_sq.sqrt() * vv).recip();
  let lat_deg = (z + e_two_sq * z_o).atan2(r) * 180.0 / PI;
  let alt_km = uu * (1.0 - z_o * z.recip()) * 0.001;
  (lat_deg, alt_km)
}
