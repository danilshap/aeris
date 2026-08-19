use serde::Deserialize;

pub const METERS_PER_LATITUDE_DEGREE: f64 = 111_320.0;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct Coordinates {
    latitude: f64,
    longitude: f64,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }
}
