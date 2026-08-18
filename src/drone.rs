mod config;
mod mission;

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
    charge: u8,
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
        charge: u8,
        config: DroneConfig,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            coordinates,
            altitude,
            speed,
            charge,
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

    pub fn tick(&mut self, delta_time: f32) -> Result<(), AerisError> {
        match self.current_task {
            Some(DroneTask::Takeoff { target_altitude }) => {
                if self.flight_mode == FlightMode::Armed {
                    self.takeoff()?;
                }

                self.altitude += delta_time * self.config.climb_speed;

                if self.altitude >= target_altitude {
                    self.altitude = target_altitude;
                    self.current_task = None;
                    self.hold()?;
                }
            }
            Some(DroneTask::ReturnHome) => {
                self.return_home()?;
                self.current_task = None;
            }
            Some(DroneTask::Land) => {
                if self.flight_mode == FlightMode::Hold
                    || self.flight_mode == FlightMode::ReturnHome
                {
                    self.land()?;
                }

                self.altitude -= delta_time * self.config.descent_speed;

                if self.altitude <= 0.0 {
                    self.altitude = 0.0;
                    self.flight_mode = FlightMode::Idle;
                    self.current_task = None;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn arm(&mut self) -> Result<(), AerisError> {
        if self.flight_mode != FlightMode::Idle {
            return Err(AerisError::InvalidFlightModeTransition {
                from: format!("{:?}", self.flight_mode),
                to: "Armed".to_string(),
            });
        }

        self.flight_mode = FlightMode::Armed;
        Ok(())
    }

    pub fn takeoff(&mut self) -> Result<(), AerisError> {
        if self.flight_mode != FlightMode::Armed {
            return Err(AerisError::InvalidFlightModeTransition {
                from: format!("{:?}", self.flight_mode),
                to: "Takeoff".to_string(),
            });
        }

        self.flight_mode = FlightMode::Takeoff;

        Ok(())
    }

    pub fn hold(&mut self) -> Result<(), AerisError> {
        if self.flight_mode != FlightMode::Takeoff && self.flight_mode != FlightMode::Mission {
            return Err(AerisError::InvalidFlightModeTransition {
                from: format!("{:?}", self.flight_mode),
                to: "Hold".to_string(),
            });
        }

        self.flight_mode = FlightMode::Hold;
        Ok(())
    }

    pub fn start_mission(&mut self) -> Result<(), AerisError> {
        if self.flight_mode != FlightMode::Takeoff && self.flight_mode != FlightMode::Hold {
            return Err(AerisError::InvalidFlightModeTransition {
                from: format!("{:?}", self.flight_mode),
                to: "Mission".to_string(),
            });
        }

        self.flight_mode = FlightMode::Mission;
        Ok(())
    }

    pub fn return_home(&mut self) -> Result<(), AerisError> {
        if self.flight_mode != FlightMode::Hold && self.flight_mode != FlightMode::Mission {
            return Err(AerisError::InvalidFlightModeTransition {
                from: format!("{:?}", self.flight_mode),
                to: "ReturnHome".to_string(),
            });
        }

        self.flight_mode = FlightMode::ReturnHome;
        Ok(())
    }

    pub fn land(&mut self) -> Result<(), AerisError> {
        if self.flight_mode != FlightMode::ReturnHome && self.flight_mode != FlightMode::Hold {
            return Err(AerisError::InvalidFlightModeTransition {
                from: format!("{:?}", self.flight_mode),
                to: "Landing".to_string(),
            });
        }

        self.flight_mode = FlightMode::Landing;

        Ok(())
    }

    pub fn emergency(&mut self) -> Result<(), AerisError> {
        self.flight_mode = FlightMode::Emergency;
        Ok(())
    }
}
