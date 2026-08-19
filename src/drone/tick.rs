use super::{ConnectionStatus, Drone, DroneTask, FlightMode};
use crate::errors::AerisError;

impl Drone {
    pub fn tick(&mut self, delta_time: f32) -> Result<(), AerisError> {
        self.update_battery(delta_time);

        if self.battery == 0.0 {
            return Ok(());
        }

        self.process_current_task(delta_time)
    }

    fn update_battery(&mut self, delta_time: f32) {
        self.battery -= self.config.consumption_per_second * delta_time;

        if self.battery <= 0.0 {
            self.battery = 0.0;
            self.current_task = None;
            self.connection_status = ConnectionStatus::Lost;
        }
    }

    fn process_current_task(&mut self, delta_time: f32) -> Result<(), AerisError> {
        match self.current_task {
            Some(DroneTask::Takeoff { target_altitude }) => {
                self.process_takeoff(target_altitude, delta_time)?;
            }
            Some(DroneTask::ReturnHome) => self.process_return_home()?,
            Some(DroneTask::Land) => self.process_landing(delta_time)?,
            _ => {}
        }

        Ok(())
    }

    fn process_takeoff(&mut self, target_altitude: f32, delta_time: f32) -> Result<(), AerisError> {
        if self.flight_mode == FlightMode::Armed {
            self.takeoff()?;
        }

        self.altitude += delta_time * self.config.climb_speed;

        if self.altitude >= target_altitude {
            self.altitude = target_altitude;
            self.current_task = None;
            self.hold()?;
        }

        Ok(())
    }

    fn process_return_home(&mut self) -> Result<(), AerisError> {
        self.return_home()?;
        self.current_task = None;

        Ok(())
    }

    fn process_landing(&mut self, delta_time: f32) -> Result<(), AerisError> {
        if self.flight_mode == FlightMode::Hold || self.flight_mode == FlightMode::ReturnHome {
            self.land()?;
        }

        self.altitude -= delta_time * self.config.descent_speed;

        if self.altitude <= 0.0 {
            self.altitude = 0.0;
            self.flight_mode = FlightMode::Idle;
            self.current_task = None;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drone::{Coordinates, DroneConfig};

    fn drone_at(altitude: f32) -> Drone {
        Drone::new(
            Coordinates {
                latitude: 0.0,
                longitude: 0.0,
            },
            altitude,
            0.0,
            100.0,
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
    fn takeoff_reaches_target_and_holds() {
        let mut drone = drone_at(0.0);
        drone.connect().unwrap();
        drone.arm().unwrap();
        drone.assign_task(Some(DroneTask::Takeoff {
            target_altitude: 10.0,
        }));

        drone.tick(10.0).unwrap();

        assert_eq!(drone.altitude(), 10.0);
        assert_eq!(drone.flight_mode(), &FlightMode::Hold);
        assert!(drone.current_task().is_none());
    }

    #[test]
    fn return_home_changes_mode_and_clears_task() {
        let mut drone = drone_at(10.0);
        drone.connect().unwrap();
        drone.arm().unwrap();
        drone.takeoff().unwrap();
        drone.hold().unwrap();
        drone.assign_task(Some(DroneTask::ReturnHome));

        drone.tick(1.0).unwrap();

        assert_eq!(drone.flight_mode(), &FlightMode::ReturnHome);
        assert!(drone.current_task().is_none());
    }

    #[test]
    fn landing_reaches_ground_and_becomes_idle() {
        let mut drone = drone_at(10.0);
        drone.connect().unwrap();
        drone.arm().unwrap();
        drone.takeoff().unwrap();
        drone.hold().unwrap();
        drone.return_home().unwrap();
        drone.assign_task(Some(DroneTask::Land));

        drone.tick(10.0).unwrap();

        assert_eq!(drone.altitude(), 0.0);
        assert_eq!(drone.flight_mode(), &FlightMode::Idle);
        assert!(drone.current_task().is_none());
    }
}
