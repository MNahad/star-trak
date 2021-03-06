use super::{Geodetic, Cartesian, Horizontal};
use std::f64::consts::PI;

pub fn eci_to_geodetic(position_eci_km: &Cartesian, gmst: f64) -> Geodetic {
  let theta = position_eci_km.y.atan2(position_eci_km.x);
  let theta = if theta < 0.0 { theta + 2.0 * PI } else { theta };
  let lambda_e = theta - gmst;
  let lambda_e = if lambda_e > PI { lambda_e - 2.0 * PI } else { lambda_e };
  let lambda_e = if lambda_e < -PI { lambda_e + 2.0 * PI } else { lambda_e };
  let lon_deg = lambda_e.to_degrees();
  let r_km = (position_eci_km.x.powi(2) + position_eci_km.y.powi(2)).sqrt();
  let (lat_deg, alt_km) = compute_geodetic_coords_2d(r_km, position_eci_km.z);
  Geodetic { lat_deg, lon_deg, alt_km }
}

pub fn geodetic_to_eci(position_geodetic: &Geodetic, gmst: f64) -> Cartesian {
  const A: f64 = 6378137.0;
  const E_SQ: f64 = 0.00669437999014;
  let cos_phi = position_geodetic.lat_deg.to_radians().cos();
  let sin_phi = position_geodetic.lat_deg.to_radians().sin();
  let cc = A / (1.0 - E_SQ * sin_phi.powi(2)).sqrt();
  let theta = gmst + position_geodetic.lon_deg.to_radians();
  Cartesian {
    x: ((cc + position_geodetic.alt_km * 1000.0) * cos_phi * theta.cos()) * 0.001,
    y: ((cc + position_geodetic.alt_km * 1000.0) * cos_phi * theta.sin()) * 0.001,
    z: ((cc * (1.0 - E_SQ) + position_geodetic.alt_km * 1000.0) * sin_phi) * 0.001,
  }
}

pub fn eci_to_topocentric_enu(
  vector_km: &Cartesian,
  lat_deg: f64,
  lon_deg: f64,
  gmst: f64,
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
    elevation_deg: (enu_km.z / range_km).asin().to_degrees(),
    range_km,
  }
}

fn compute_geodetic_coords_2d(r_km: f64, z_km: f64) -> (f64, f64) {
  // Refer to
  // J. Zhu, "Conversion of Earth-centered Earth-fixed coordinates to geodetic coordinates,"
  // IEEE Transactions on Aerospace and Electronic Systems, vol 30, pp 957-961, 1994.
  const A_SQ: f64 = 40680631590769.0;
  const B_SQ: f64 = 40408299984087.0;
  const E_SQ: f64 = 0.00669437999014;
  const E_TWO_SQ: f64 = 0.00673949674228;
  let r = r_km * 1000.0;
  let r_sq = r.powi(2);
  let z = z_km * 1000.0;
  let z_sq = z.powi(2);
  let ee_sq = A_SQ - B_SQ;
  let ff = 54.0 * B_SQ * z_sq;
  let gg = r_sq + ((1.0 - E_SQ) * z_sq) - (E_SQ * ee_sq);
  let cc = (E_SQ.powi(2) * ff * r_sq) / gg.powi(3);
  let ss = (1.0 + cc + (cc.powi(2) + 2.0 * cc).sqrt()).cbrt();
  let pp = ff / (3.0 * (ss + ss.recip() + 1.0).powi(2) * gg.powi(2));
  let qq = (1.0 + 2.0 * E_SQ.powi(2) * pp).sqrt();
  let r_o = ((-(pp * E_SQ * r)) / (1.0 + qq)) +
    (
      (0.5 * A_SQ * (1.0 + qq.recip()))
      - ((pp * (1.0 - E_SQ) * z_sq) / (qq * (1.0 + qq)))
      - (0.5 * pp * r_sq)
    ).sqrt();
  let uu = ((r - E_SQ * r_o).powi(2) + z_sq).sqrt();
  let vv = ((r - E_SQ * r_o).powi(2) + (1.0 - E_SQ) * z_sq).sqrt();
  let z_o = B_SQ * z / (A_SQ.sqrt() * vv);
  let lat_deg = (z + E_TWO_SQ * z_o).atan2(r).to_degrees();
  let alt_km = uu * (1.0 - z_o / z) * 0.001;
  (lat_deg, alt_km)
}

#[cfg(test)]
mod tests {
  use super::{Geodetic, Horizontal, Cartesian};

  fn assert_eq(left: f64, right: f64) {
    assert!((left - right).abs() <= 1e-6);
  }

  #[test]
  fn it_converts_eci_to_geodetic() {
    let eci = Cartesian {
      x: -1040.847789,
      y: 6686.458279,
      z: 3670.305288,
    };
    let gmst = 3.132033;
    let geodetic = Geodetic {
      lat_deg: 28.608389,
      lon_deg: -80.604333,
      alt_km: 1325.000000,
    };
    let geo_result = super::eci_to_geodetic(&eci, gmst);
    assert_eq(geo_result.lat_deg, geodetic.lat_deg);
    assert_eq(geo_result.lon_deg, geodetic.lon_deg);
    assert_eq(geo_result.alt_km, geodetic.alt_km);
  }

  #[test]
  fn it_converts_geodetic_to_eci() {
    let geodetic = Geodetic {
      lat_deg: 28.608389,
      lon_deg: -80.604333,
      alt_km: 1325.000000,
    };
    let gmst = 3.132033;
    let eci = Cartesian {
      x: -1040.847789,
      y: 6686.458279,
      z: 3670.305288,
    };
    let eci_result = super::geodetic_to_eci(&geodetic, gmst);
    assert_eq(eci_result.x, eci.x);
    assert_eq(eci_result.y, eci.y);
    assert_eq(eci_result.z, eci.z);
  }

  #[test]
  fn it_converts_eci_to_aer() {
    let eci = Cartesian {
      x: -1040.847789 - 2558.531817,
      y: 6686.458279 - 4639.454865,
      z: 3670.305288 - 3539.150874,
    };
  let observer_lat_lng = [33.920700, -118.327800];
    let gmst = 3.132033;
    let aer = Horizontal {
      azimuth_deg: 88.913626,
      elevation_deg: 1.635970,
      range_km: 4142.820054,
    };
    let aer_result = super::enu_to_aer(&super::eci_to_topocentric_enu(
      &eci,
      observer_lat_lng[0],
      observer_lat_lng[1],
      gmst,
    ));
    assert_eq(aer_result.azimuth_deg, aer.azimuth_deg);
    assert_eq(aer_result.elevation_deg, aer.elevation_deg);
    assert_eq(aer_result.range_km, aer.range_km);
  }
}
