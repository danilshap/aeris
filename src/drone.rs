mod config;
mod flight;
mod tick;

use crate::coordinates::Coordinates;
use crate::errors::AerisError;
use crate::mission::DroneTask;
use uuid::Uuid;

pub use config::DroneConfig;

#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Lost,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FlightMode {
    Idle,
    Armed,
    Takeoff,
    Hold,
    Mission,
    ReturnToHome,
    Landing,
}

#[derive(Debug, Clone)]
pub struct Drone {
    id: Uuid,
    name: String,
    coordinates: Coordinates,
    home_position: Coordinates,
    flight_start_position: Coordinates,
    altitude: f32,
    speed: f32,
    battery_charge: f32,
    connection_status: ConnectionStatus,
    flight_mode: FlightMode,
    config: DroneConfig,
    current_task: Option<DroneTask>,
}

impl Drone {
    pub fn new(
        name: String,
        home_position: Coordinates,
        altitude: f32,
        speed: f32,
        config: DroneConfig,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            coordinates: home_position,
            home_position,
            flight_start_position: home_position,
            altitude,
            speed,
            battery_charge: config.battery_capacity,
            connection_status: ConnectionStatus::Disconnected,
            flight_mode: FlightMode::Idle,
            config,
            current_task: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn altitude(&self) -> f32 {
        self.altitude
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn coordinates(&self) -> &Coordinates {
        &self.coordinates
    }

    pub fn home_position(&self) -> &Coordinates {
        &self.home_position
    }

    pub fn flight_start_position(&self) -> &Coordinates {
        &self.flight_start_position
    }

    pub fn flight_mode(&self) -> &FlightMode {
        &self.flight_mode
    }

    pub fn connection_status(&self) -> &ConnectionStatus {
        &self.connection_status
    }

    pub fn current_task(&self) -> Option<&DroneTask> {
        self.current_task.as_ref()
    }

    pub fn battery_percentage(&self) -> f32 {
        let percentage = self.battery_charge / self.config.battery_capacity * 100.0;

        percentage.clamp(0.0, 100.0)
    }

    pub fn connect(&mut self) -> Result<(), AerisError> {
        match self.connection_status {
            ConnectionStatus::Connecting => Err(AerisError::UnexeptbleState {
                action: String::from("connect"),
                state: format!("{:?}", self.connection_status),
            }),
            ConnectionStatus::Connected => Ok(()),
            ConnectionStatus::Disconnected | ConnectionStatus::Lost => {
                self.connection_status = ConnectionStatus::Connecting;
                // todo: timer for connection
                self.connection_status = ConnectionStatus::Connected;
                Ok(())
            }
        }
    }

    pub fn assign_task(&mut self, new_task: Option<DroneTask>) {
        if new_task.is_some() {
            self.flight_start_position = self.coordinates;
        }

        self.current_task = new_task;
    }
}
