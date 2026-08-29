use super::{ConnectionStatus, Drone, FlightMode};
use crate::errors::AerisError;

impl Drone {
    pub fn arm(&mut self) -> Result<(), AerisError> {
        if self.connection_status != ConnectionStatus::Connected {
            return Err(AerisError::UnexpectedState {
                action: "arm".to_string(),
                state: format!("{:?}", self.connection_status),
            });
        }

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
                to: "ReturnToHome".to_string(),
            });
        }

        self.flight_mode = FlightMode::ReturnToHome;
        Ok(())
    }

    pub fn land(&mut self) -> Result<(), AerisError> {
        if self.flight_mode != FlightMode::ReturnToHome && self.flight_mode != FlightMode::Hold {
            return Err(AerisError::InvalidFlightModeTransition {
                from: format!("{:?}", self.flight_mode),
                to: "Landing".to_string(),
            });
        }

        self.flight_mode = FlightMode::Landing;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coordinates::Coordinates, drone::DroneConfig};

    fn drone() -> Drone {
        Drone::new(
            "DR-TEST-01".to_string(),
            Coordinates::new(0.0, 0.0),
            0.0,
            0.0,
            DroneConfig {
                name: "test".to_string(),
                max_speed: 10.0,
                climb_speed: 1.0,
                descent_speed: 1.0,
                max_altitude: 100.0,
                battery_capacity: 100.0,
                consumption_per_second: 0.0,
            },
        )
    }

    #[test]
    fn normal_flight_reaches_landing_mode() {
        let mut drone = drone();

        drone.connect().unwrap();
        drone.arm().unwrap();
        assert_eq!(drone.flight_mode(), &FlightMode::Armed);

        drone.takeoff().unwrap();
        assert_eq!(drone.flight_mode(), &FlightMode::Takeoff);

        drone.hold().unwrap();
        assert_eq!(drone.flight_mode(), &FlightMode::Hold);

        drone.start_mission().unwrap();
        assert_eq!(drone.flight_mode(), &FlightMode::Mission);

        drone.return_home().unwrap();
        assert_eq!(drone.flight_mode(), &FlightMode::ReturnToHome);

        drone.land().unwrap();
        assert_eq!(drone.flight_mode(), &FlightMode::Landing);
    }

    #[test]
    fn invalid_transition_keeps_current_mode() {
        let mut drone = drone();

        let result = drone.takeoff();

        assert!(matches!(
            result,
            Err(AerisError::InvalidFlightModeTransition { from, to })
                if from == "Idle" && to == "Takeoff"
        ));
        assert_eq!(drone.flight_mode(), &FlightMode::Idle);
    }

    #[test]
    fn arm_requires_connection() {
        let mut drone = drone();

        let result = drone.arm();

        assert!(matches!(
            result,
            Err(AerisError::UnexpectedState { action, state })
                if action == "arm" && state == "Disconnected"
        ));
        assert_eq!(drone.flight_mode(), &FlightMode::Idle);
    }
}
