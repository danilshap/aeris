use std::fs;

use crate::{
    app_config::{AppConfig, DroneCatalog},
    errors::AerisError,
};

pub fn load_drone_catalog(path: &str) -> Result<DroneCatalog, AerisError> {
    let content = fs::read_to_string(path)?;
    let catalog = toml::from_str(&content)?;

    Ok(catalog)
}

pub fn load_app_config(path: &str) -> Result<AppConfig, AerisError> {
    let content = fs::read_to_string(path)?;
    let config = toml::from_str(&content)?;

    Ok(config)
}
