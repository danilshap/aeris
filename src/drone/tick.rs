use super::{ConnectionStatus, Drone, FlightMode};
use crate::{
    coordinates::{Coordinates, METERS_PER_LATITUDE_DEGREE},
    errors::AerisError,
    mission::DroneTask,
};

impl Drone {
    pub fn tick(&mut self, delta_time: f32) -> Result<(), AerisError> {
        self.update_battery(delta_time);

        if self.battery_charge == 0.0 {
            return Ok(());
        }

        self.process_current_task(delta_time)
    }

    fn update_battery(&mut self, delta_time: f32) {
        let consumed = self.config.consumption_per_second * delta_time;
        self.battery_charge = (self.battery_charge - consumed).max(0.0);

        if self.battery_charge <= 0.0 {
            self.speed = 0.0;
            self.current_task = None;
            self.connection_status = ConnectionStatus::Lost;
        }
    }

    fn process_current_task(&mut self, delta_time: f32) -> Result<(), AerisError> {
        match self.current_task {
            Some(DroneTask::Takeoff { target_altitude }) => {
                self.process_takeoff(target_altitude, delta_time)?
            }
            Some(DroneTask::FlyTo { target }) => self.process_fly_to(target, delta_time)?,
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

    fn process_fly_to(&mut self, target: Coordinates, delta_time: f32) -> Result<(), AerisError> {
        let returning_home = target == self.home_position;

        if returning_home && self.flight_mode != FlightMode::ReturnToHome {
            self.return_home()?;
        } else if !returning_home && self.flight_mode != FlightMode::Mission {
            self.start_mission()?;
        }

        self.speed = self.config.max_speed;

        let current_latitude = self.coordinates.latitude();
        let current_longitude = self.coordinates.longitude();

        let latitude_delta = target.latitude() - current_latitude;
        let longitude_delta = target.longitude() - current_longitude;
        let mean_latitude = (current_latitude + target.latitude()) / 2.0;

        let latitude_distance = latitude_delta * METERS_PER_LATITUDE_DEGREE;
        let longitude_distance =
            longitude_delta * METERS_PER_LATITUDE_DEGREE * mean_latitude.to_radians().cos();

        let remaining_distance = latitude_distance.hypot(longitude_distance);
        let movement_distance = f64::from(self.speed * delta_time);

        if remaining_distance <= movement_distance {
            self.coordinates = target;
            self.speed = 0.0;
            self.current_task = None;

            if !returning_home {
                self.hold()?;
            }

            return Ok(());
        }

        let movement_ratio = movement_distance / remaining_distance;

        self.coordinates = Coordinates::new(
            current_latitude + latitude_delta * movement_ratio,
            current_longitude + longitude_delta * movement_ratio,
        );

        Ok(())
    }

    fn process_return_home(&mut self) -> Result<(), AerisError> {
        self.return_home()?;
        self.current_task = None;

        Ok(())
    }

    fn process_landing(&mut self, delta_time: f32) -> Result<(), AerisError> {
        if self.flight_mode == FlightMode::Hold || self.flight_mode == FlightMode::ReturnToHome {
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
            "DR-TEST-01".to_string(),
            Coordinates::new(0.0, 0.0),
            altitude,
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

        assert_eq!(drone.flight_mode(), &FlightMode::ReturnToHome);
        assert!(drone.current_task().is_none());
    }

    #[test]
    fn fly_to_home_uses_return_to_home_mode() {
        let mut drone = drone_at(10.0);
        drone.coordinates = Coordinates::new(0.0, 0.001);
        drone.connect().unwrap();
        drone.arm().unwrap();
        drone.takeoff().unwrap();
        drone.hold().unwrap();

        let home_position = *drone.home_position();
        drone.assign_task(Some(DroneTask::FlyTo {
            target: home_position,
        }));

        drone.tick(0.1).unwrap();

        assert_eq!(drone.flight_mode(), &FlightMode::ReturnToHome);
        assert!(drone.current_task().is_some());
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

    #[test]
    fn battery_charge_decreases_from_current_charge() {
        let mut drone = drone_at(0.0);
        drone.config.consumption_per_second = 2.0;

        drone.tick(3.0).unwrap();
        drone.tick(2.0).unwrap();

        assert_eq!(drone.battery_percentage(), 90.0);
    }

    #[test]
    fn depleted_battery_stops_drone_and_loses_connection() {
        let mut drone = drone_at(10.0);
        drone.config.consumption_per_second = 60.0;
        drone.speed = drone.config.max_speed;
        drone.assign_task(Some(DroneTask::Land));

        drone.tick(2.0).unwrap();

        assert_eq!(drone.battery_percentage(), 0.0);
        assert_eq!(drone.speed(), 0.0);
        assert_eq!(drone.connection_status(), &ConnectionStatus::Lost);
        assert!(drone.current_task().is_none());
    }

    #[test]
    fn fly_to_uses_max_speed_and_stops_at_target() {
        let mut drone = drone_at(10.0);
        drone.connect().unwrap();
        drone.arm().unwrap();
        drone.takeoff().unwrap();
        drone.hold().unwrap();

        let target = Coordinates::new(0.0, 0.001);
        drone.assign_task(Some(DroneTask::FlyTo { target }));

        drone.tick(0.1).unwrap();

        assert_eq!(drone.speed(), 10.0);
        assert_ne!(drone.coordinates(), &target);

        drone.tick(20.0).unwrap();

        assert_eq!(drone.speed(), 0.0);
        assert_eq!(drone.coordinates(), &target);
    }
}
