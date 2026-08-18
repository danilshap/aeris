use serde::Deserialize;

use crate::drone::DroneConfig;
use crate::drone::DroneTask;

#[derive(Debug, Deserialize)]
pub struct MissionConfig {
    pub name: String,
    pub groups: Vec<MissionGroupConfig>,
}

#[derive(Debug, Deserialize)]
pub struct MissionGroupConfig {
    pub drone_type: String,
    pub count: usize,
    pub tasks: Vec<DroneTask>,
}

#[derive(Debug, Deserialize)]
pub struct DroneCatalog {
    pub drones: Vec<DroneConfig>,
}
