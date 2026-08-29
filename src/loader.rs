use std::fs;

use crate::{drone::DroneCatalog, errors::AerisError, mission::MissionConfig};

pub fn load_drone_catalog(path: &str) -> Result<DroneCatalog, AerisError> {
    let content = fs::read_to_string(path)?;
    let catalog = toml::from_str(&content)?;

    Ok(catalog)
}

pub fn load_mission_config(path: &str) -> Result<MissionConfig, AerisError> {
    let content = fs::read_to_string(path)?;
    let config = toml::from_str(&content)?;

    Ok(config)
}
