mod config;
mod flight;
mod mission;
mod tick;

use crate::errors::AerisError;
use uuid::Uuid;

pub use config::DroneConfig;
pub use mission::DroneTask;
pub use mission::Mission;

#[derive(Debug)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, PartialEq)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Lost,
}

#[derive(Debug, PartialEq)]
pub enum FlightMode {
    Idle,
    Armed,
    Takeoff,
    Hold,
    Mission,
    ReturnHome,
    Landing,
    Emergency,
}

#[derive(Debug)]
pub struct Drone {
    id: Uuid,
    coordinates: Coordinates,
    altitude: f32,
    speed: f32,
    battery: f32,
    connection_status: ConnectionStatus,
    flight_mode: FlightMode,
    config: DroneConfig,
    current_task: Option<DroneTask>,
}

impl Drone {
    pub fn new(
        coordinates: Coordinates,
        altitude: f32,
        speed: f32,
        battery: f32,
        config: DroneConfig,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            coordinates,
            altitude,
            speed,
            battery,
            connection_status: ConnectionStatus::Disconnected,
            flight_mode: FlightMode::Idle,
            config,
            current_task: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn altitude(&self) -> f32 {
        self.altitude
    }

    pub fn flight_mode(&self) -> &FlightMode {
        &self.flight_mode
    }

    pub fn current_task(&self) -> Option<&DroneTask> {
        self.current_task.as_ref()
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

    pub fn disconnect(&mut self) -> Result<(), AerisError> {
        match self.connection_status {
            ConnectionStatus::Disconnected => Ok(()),
            _ => {
                // todo: timer for disconnection
                self.connection_status = ConnectionStatus::Disconnected;
                Ok(())
            }
        }
    }

    pub fn assign_task(&mut self, new_task: Option<DroneTask>) {
        self.current_task = new_task;
    }
}
