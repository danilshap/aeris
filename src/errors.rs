use thiserror::Error;

#[derive(Error, Debug)]
pub enum AerisError {
    #[error("Cant process {action} with {state} state")]
    UnexeptbleState { action: String, state: String },

    #[error("Cannot change flight mode from {from} to {to}")]
    InvalidFlightModeTransition { from: String, to: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse TOML config: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Drone type '{0}' not found in drone catalog")]
    DroneTypeNotFound(String),

    #[error("Invalid mission. Reason: {0}")]
    InvalidMission(String),
}
