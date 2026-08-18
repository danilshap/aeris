use std::fs;

use crate::{
    errors::AerisError,
    mission_config::{DroneCatalog, MissionConfig},
};

pub fn load_drone_catalog(path: &str) -> Result<DroneCatalog, AerisError> {
    let content = fs::read_to_string(path)?;
    let catalog = toml::from_str(&content)?;

    Ok(catalog)
}

pub fn load_mission_config(path: &str) -> Result<MissionConfig, AerisError> {
    let content = std::fs::read_to_string(path)?;
    let config = toml::from_str(&content)?;

    Ok(config)
}
