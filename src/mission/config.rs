use serde::Deserialize;

use crate::{coordinates::Coordinates, drone::DroneTask};

#[derive(Debug, Deserialize)]
pub struct MissionConfig {
    pub name: String,
    #[serde(default)]
    pub random_failure: bool,
    pub groups: Vec<MissionGroupConfig>,
}

#[derive(Debug, Deserialize)]
pub struct MissionGroupConfig {
    pub drone_type: String,
    pub drone_names: Vec<String>,
    pub home_position: Coordinates,
    pub tasks: Vec<DroneTask>,
}
