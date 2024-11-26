/// Simulated models of vehicle movement used to generate a stream of telemetry / global
/// position messages.
///
/// Initial (demo) version is not fully tested outside of northern hemisphere, west of prime
/// meridian.
use std::time::Duration;

use tracing::debug;

use crate::{
    coord::{BBoxWGS, Heading, Point2d},
    protocol::TelemMsg,
};

//  _____
// |_   _|   _ _ __   ___  ___
//   | || | | | '_ \ / _ \/ __|
//   | || |_| | |_) |  __/\__ \
//   |_| \__, | .__/ \___||___/
//       |___/|_|

/// API for stream of telemetry messages.
pub trait TelemStream<M>
where
    M: TelemMsg,
{
    fn next(&mut self, delta_t: TimeDelta) -> M;
}

/// Time delta since last telemetry message, with [`Default`] and conversion from [`Duration`] for
/// convenience.
pub struct TimeDelta {
    msec: u32,
}

impl Default for TimeDelta {
    fn default() -> Self {
        Self { msec: 1000 }
    }
}

impl From<Duration> for TimeDelta {
    fn from(d: Duration) -> Self {
        Self {
            msec: d.as_millis() as u32,
        }
    }
}

impl TimeDelta {
    pub fn seconds(&self) -> f32 {
        self.msec as f32 / 1000.0
    }
}

//  ____  _                 _        __  __           _      _
// / ___|(_)_ __ ___  _ __ | | ___  |  \/  | ___   __| | ___| |
// \___ \| | '_ ` _ \| '_ \| |/ _ \ | |\/| |/ _ \ / _` |/ _ \ |
//  ___) | | | | | | | |_) | |  __/ | |  | | (_) | (_| |  __/ |
// |____/|_|_| |_| |_| .__/|_|\___| |_|  |_|\___/ \__,_|\___|_|
//                   |_|

/// Super simple vehicle motion model. Not realistic.
pub struct RandomWalk {
    bbox: BBoxWGS,
    max_velocity_mps: f32,
    pub(crate) last_pos: Point2d,
    heading: Heading,
}

impl RandomWalk {
    pub fn new(bbox: BBoxWGS, max_velocity_mps: f32) -> Self {
        let start_pos = bbox.midpoint();
        let random_deg = rand::random::<f32>() * 360.0;
        Self {
            bbox,
            max_velocity_mps,
            last_pos: start_pos,
            heading: Heading(random_deg),
        }
    }
}

impl<M> TelemStream<M> for RandomWalk
where
    M: TelemMsg,
{
    fn next(&mut self, delta_t: TimeDelta) -> M {
        let vel = rand::random::<f32>() * self.max_velocity_mps;

        let turn = (rand::random::<f32>() - 0.5) * 10.0;
        self.heading.rot(turn);
        debug!("heading after {} turn: {}", turn, self.heading.0);

        // calculate meters to move then convert to degrees
        let dist_m = (delta_t.seconds() * vel) as f64;
        let delta_x = dist_m * self.heading.to_radians().cos() as f64;
        let delta_y = dist_m * self.heading.to_radians().sin() as f64;
        let delta_lat = delta_y / BBoxWGS::meter_per_deg_lat(self.last_pos.0).unwrap();
        let delta_lon = delta_x / BBoxWGS::meter_per_deg_lon(self.last_pos.0).unwrap();
        debug!(
            "delta_lat: {:.4} ({:.2} m), delta_lon: {:.4} ({:.2} m)",
            delta_lat, delta_y, delta_lon, delta_x
        );
        let new_lat = self.last_pos.0 + delta_lat;
        let new_lon = self.last_pos.1 + delta_lon;

        // TODO math that works outside of N hemisphere, etc.
        // clamp to bbox
        let mut oob = false;
        let new_lat = if new_lat > self.bbox.upper_left.0 {
            oob = true;
            debug!("lat oob: {} -> {}", new_lat, self.bbox.upper_left.0);
            self.bbox.upper_left.0
        } else if new_lat < self.bbox.lower_right.0 {
            oob = true;
            debug!("lat oob: {} -> {}", new_lat, self.bbox.lower_right.0);
            self.bbox.lower_right.0
        } else {
            new_lat
        };
        let new_lon = if new_lon < self.bbox.upper_left.1 {
            oob = true;
            debug!("lon oob: {} -> {}", new_lon, self.bbox.upper_left.1);
            self.bbox.upper_left.1
        } else if new_lon > self.bbox.lower_right.1 {
            oob = true;
            debug!("lon oob: {} -> {}", new_lon, self.bbox.lower_right.1);
            self.bbox.lower_right.1
        } else {
            new_lon
        };
        if oob {
            self.heading.rot(180.0);
        }
        self.last_pos = Point2d(new_lat, new_lon);
        M::from_coords(new_lat, new_lon, 0.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{lazy_init_tracing, protocol::cot::CotXml};
    use tracing::trace;

    #[test]
    fn test_random_walk() {
        lazy_init_tracing();
        let bbox = BBoxWGS {
            upper_left: Point2d(39.0, -110.0),
            lower_right: Point2d(38.0, -109.5),
        };
        let mut rw = RandomWalk::new(bbox, 100.0);
        for _ in 0..100 {
            let msg: CotXml = rw.next(Duration::new(60, 0).into());
            trace!("msg: {:?}", msg);
            let Point2d(lat, lon) = rw.last_pos;
            debug!("lat: {:.5}, lon: {:.5}", lat, lon);
            assert!((38.0..=39.0).contains(&lat));
            assert!((-110.0..=-109.5).contains(&lon));
        }
    }
}
