/// Coordinate types and utilities.
use crate::{Error, TGResult};

//  _____
// |_   _|   _ _ __   ___  ___
//   | || | | | '_ \ / _ \/ __|
//   | || |_| | |_) |  __/\__ \
//   |_| \__, | .__/ \___||___/
//       |___/|_|

#[derive(Debug, Copy, Clone)]
pub struct Point2d(pub f64, pub f64);

/// Bounding box rectangle with corners in WGS 84 coordinates.
pub struct BBoxWGS {
    pub upper_left: Point2d,
    pub lower_right: Point2d,
}

impl BBoxWGS {
    pub fn new(upper_left: Point2d, lower_right: Point2d) -> TGResult<Self> {
        Self::validate_lat_lon(upper_left)?;
        Self::validate_lat_lon(lower_right)?;
        Ok(Self {
            upper_left,
            lower_right,
        })
    }

    pub fn midpoint(&self) -> Point2d {
        Point2d(
            (self.upper_left.0 + self.lower_right.0) / 2.0,
            (self.upper_left.1 + self.lower_right.1) / 2.0,
        )
    }

    /// Return approximate (width, height) in meters.
    pub fn approx_dimensions_m(&self) -> TGResult<Point2d> {
        // Use midpoint latitude as an approximate input for meters per degree calcs.
        let Point2d(mid_lat, _mid_lon) = self.midpoint();
        let lat_deg = self.lower_right.0 - self.upper_left.0;
        let lon_deg = self.lower_right.1 - self.upper_left.1;
        let width = Self::meter_per_deg_lon(mid_lat)? * lon_deg;
        let height = Self::meter_per_deg_lat(mid_lat)? * lat_deg;
        Ok(Point2d(width.abs(), height.abs()))
    }

    fn validate_lat_lon(coord: Point2d) -> TGResult<()> {
        Self::validate_lat(coord.0)?;
        Self::validate_lon(coord.1)?;
        Ok(())
    }

    fn validate_lat(lat: f64) -> TGResult<()> {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(Error::InvalidCoord(format!("latitude {lat}")));
        }
        Ok(())
    }

    fn validate_lon(lon: f64) -> TGResult<()> {
        if !(-180.0..=180.0).contains(&lon) {
            return Err(Error::InvalidCoord(format!("longitude {lon}")));
        }
        Ok(())
    }

    pub fn meter_per_deg_lat(lat_deg: f64) -> TGResult<f64> {
        Self::validate_lat(lat_deg)?;
        // num meters to travel 1 degree on N-S line changes with latitude:
        // 111132.92 - 559.82 * cos(2 * lat_rad) + 1.175 * cos(4 * lat_rad) - 0.0023 * cos(6 * lat_rad)
        // Source: https://en.wikipedia.org/wiki/Geographic_coordinate_system
        let lat_rad = lat_deg.to_radians();
        Ok(
            111132.92 - 559.82 * (2.0 * lat_rad).cos() + 1.175 * (4.0 * lat_rad).cos()
                - 0.0023 * (6.0 * lat_rad).cos(),
        )
    }

    pub fn meter_per_deg_lon(lat_deg: f64) -> TGResult<f64> {
        Self::validate_lat(lat_deg)?;
        let lat_rad = lat_deg.to_radians();

        // Meters per degree of longitude on an E-W line depends on the circle of latitude.
        // m = 111412.84 * cos(lat_rad) - 93.5 * cos(3 * lat_rad) + 0.118 * cos(5 * lat_rad)
        Ok(
            111412.84 * lat_rad.cos() - 93.5 * (3.0 * lat_rad).cos()
                + 0.118 * (5.0 * lat_rad).cos(),
        )
    }
}

pub struct Heading(pub f32);

impl Heading {
    pub fn rot(&mut self, deg_cw: f32) {
        self.0 += deg_cw;
        if self.0 < 0.0 {
            self.0 += 360.0;
        } else if self.0 >= 360.0 {
            self.0 -= 360.0;
        }
    }
    pub fn to_radians(&self) -> f32 {
        self.0.to_radians()
    }
}

impl From<f32> for Heading {
    fn from(deg: f32) -> Self {
        Self(deg)
    }
}

impl From<Heading> for f32 {
    fn from(h: Heading) -> f32 {
        h.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lazy_init_tracing;

    // Test cases from Google Earth, in a "perfect rectangle", i.e.
    // Point A 40.7467136, -118.8088034
    // Point B 40.7467136, -118.3692927
    // Point C 40.3349884, -118.8088034
    // Point D 40.3349884, -118.3692927
    //
    //     A                  B
    //
    //
    //
    //     C                  D
    //
    // When projected onto globe in N hemisphere, the distancd from C to D is greater than from A
    // to B, since meters per degree longitude increases as we approach equator, i.e.
    //
    //      A                B
    //
    //
    //
    //     C                  D
    //
    // Point A to B distance (width):   ~37119m
    // Point A to C distance (height):  ~45720.5
    const PT_A: Point2d = Point2d(40.7467136, -118.8088034);
    const PT_B: Point2d = Point2d(40.7467136, -118.3692927);
    const PT_C: Point2d = Point2d(40.3349884, -118.8088034);
    const PT_D: Point2d = Point2d(40.3349884, -118.3692927);

    #[test]
    pub fn test_bbox_dim_horizontal_line() {
        lazy_init_tracing();
        let bbox = BBoxWGS::new(PT_A, PT_B).unwrap();
        let Point2d(width, height) = bbox.approx_dimensions_m().unwrap();
        // assert height, width are within one meter
        assert!((37118.0..37120.0).contains(&width));
        assert!((0.0..1.0).contains(&height));
    }

    #[test]
    pub fn test_bbox_dim_vertical_line() {
        lazy_init_tracing();
        let bbox = BBoxWGS::new(PT_A, PT_C).unwrap();
        let Point2d(width, height) = bbox.approx_dimensions_m().unwrap();
        // assert height, width are within one meter
        assert!((0.0..1.0).contains(&width));
        assert!((45719.5..45721.5).contains(&height));
    }

    #[test]
    pub fn test_bbox_dim() {
        lazy_init_tracing();
        let bbox = BBoxWGS::new(PT_A, PT_D).unwrap();
        let Point2d(width, height) = bbox.approx_dimensions_m().unwrap();
        // assert height within one meter
        // assert width up to 150m larger than A - B distance (see comments above about approaching
        // equator)
        assert!((37119.0..37269.0).contains(&width));
        assert!((45719.5..45721.5).contains(&height));
    }
}
