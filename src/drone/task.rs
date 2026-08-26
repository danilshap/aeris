use serde::Deserialize;

use crate::coordinates::Coordinates;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum DroneTask {
    Takeoff { target_altitude: f32 },
    Hold,
    FlyTo { target: Coordinates },
    ReturnHome,
    Land,
}
