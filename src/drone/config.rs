use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DroneConfig {
    pub name: String,
    pub max_speed: f32,
    pub climb_speed: f32,
    pub descent_speed: f32,
    pub max_altitude: f32,
    pub battery_capacity: f32,
    pub consumption_per_second: f32,
}

#[derive(Debug, Deserialize)]
pub struct DroneCatalog {
    pub drones: Vec<DroneConfig>,
}
