use std::f64::consts::PI;

pub struct Geodetic {
  pub lat_deg: f64,
  pub lon_deg: f64,
  pub alt_km: f64,
}

pub struct Cartesian {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

pub fn eci_to_geodetic(
  position_eci_km: &Cartesian,
  gmst: &f64,
) -> Geodetic {
  let theta = position_eci_km.y.atan2(position_eci_km.x);
  let theta = if theta < 0.0 { theta + 2.0 * PI } else { theta };
  let lambda_e = theta - gmst;
  let lambda_e = if lambda_e > PI { lambda_e - 2.0 * PI } else { lambda_e };
  let lambda_e = if lambda_e < -PI { lambda_e + 2.0 * PI } else { lambda_e };
  let lon_deg = lambda_e.to_degrees();
  let r_km = (position_eci_km.x.powi(2) + position_eci_km.y.powi(2)).sqrt();
  let (lat_deg, alt_km) = compute_geodetic_coords_2d(&r_km, &position_eci_km.z);
  Geodetic {
    lat_deg,
    lon_deg,
    alt_km,
  }
}

pub fn geodetic_to_eci(position_geodetic: &Geodetic, gmst: &f64) -> Cartesian {
  let a = 6378137.0_f64;
  let e_sq = 0.00669437999014_f64;
  let cos_phi = position_geodetic.lat_deg.to_radians().cos();
  let sin_phi = position_geodetic.lat_deg.to_radians().sin();
  let cc = (
    1.0 - e_sq * sin_phi.powi(2)
  ).sqrt().recip();
  let theta = gmst + position_geodetic.lon_deg.to_radians();
  // let theta = if theta > 2.0 * PI { theta - 2.0 * PI } else { theta };
  // let theta = if theta < 0.0 { theta + 2.0 * PI } else { theta };
  Cartesian {
    x: (a * cc + position_geodetic.alt_km) * cos_phi * theta.cos(),
    y: (a * cc + position_geodetic.alt_km) * cos_phi * theta.sin(),
    z: (a * cc * (1.0 - e_sq) + position_geodetic.alt_km) * sin_phi,
  }
}

fn compute_geodetic_coords_2d(r_km: &f64, z_km: &f64) -> (f64, f64) {
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
  let lat_deg = (z + e_two_sq * z_o).atan2(r).to_degrees();
  let alt_km = uu * (1.0 - z_o * z.recip()) * 0.001;
  (lat_deg, alt_km)
}
