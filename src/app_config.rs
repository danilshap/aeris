use serde::Deserialize;

use crate::drone::DroneConfig;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub fleet: Vec<FleetConfig>,
}

#[derive(Debug, Deserialize)]
pub struct FleetConfig {
    pub drone_type: String,
    pub count: usize,
}

#[derive(Debug, Deserialize)]
pub struct DroneCatalog {
    pub drones: Vec<DroneConfig>,
}
