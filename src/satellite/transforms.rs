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

pub struct Horizontal {
  pub azimuth_deg: f64,
  pub elevation_deg: f64,
  pub range_km: f64,
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
  let cc = a * (1.0 - e_sq * sin_phi.powi(2)).sqrt().recip();
  let theta = gmst + position_geodetic.lon_deg.to_radians();
  Cartesian {
    x: ((cc + position_geodetic.alt_km * 1000.0) * cos_phi * theta.cos()) * 0.001,
    y: ((cc + position_geodetic.alt_km * 1000.0) * cos_phi * theta.sin()) * 0.001,
    z: ((cc * (1.0 - e_sq) + position_geodetic.alt_km * 1000.0) * sin_phi) * 0.001,
  }
}

pub fn eci_to_topocentric_enu(
  vector_km: &Cartesian,
  lat_deg: &f64,
  lon_deg: &f64,
  gmst: &f64,
) -> Cartesian {
  let theta = gmst + lon_deg.to_radians();
  let s_lat = lat_deg.to_radians().sin();
  let s_lon = theta.sin();
  let c_lat = lat_deg.to_radians().cos();
  let c_lon = theta.cos();
  let m: [[f64; 3]; 3] = [
    [-s_lon, c_lon, 0.0],
    [-s_lat * c_lon, -s_lat * s_lon, c_lat],
    [c_lat * c_lon, c_lat * s_lon, s_lat],
  ];
  Cartesian {
    x: m[0][0] * vector_km.x + m[0][1] * vector_km.y + m[0][2] * vector_km.z,
    y: m[1][0] * vector_km.x + m[1][1] * vector_km.y + m[1][2] * vector_km.z,
    z: m[2][0] * vector_km.x + m[2][1] * vector_km.y + m[2][2] * vector_km.z,
  }
}

pub fn enu_to_aer(enu_km: &Cartesian) -> Horizontal {
  let range_km = (enu_km.x.powi(2) + enu_km.y.powi(2) + enu_km.z.powi(2)).sqrt();
  Horizontal {
    azimuth_deg: (enu_km.x).atan2(enu_km.y).to_degrees(),
    elevation_deg: (enu_km.z * range_km.recip()).asin().to_degrees(),
    range_km,
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