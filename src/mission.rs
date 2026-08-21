use serde::Deserialize;

mod validator;

use crate::{
    coordinates::Coordinates,
    drone::{Drone, FlightMode},
    errors::AerisError,
};

pub use validator::MissionValidator;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum DroneTask {
    Takeoff { target_altitude: f32 },
    Hold,
    FlyTo { target: Coordinates },
    ReturnHome,
    Land,
}

#[derive(Debug, PartialEq)]
pub enum MissionState {
    Ready,
    Running,
    Paused,
    Finished,
}

#[derive(Debug)]
pub struct MissionDrone {
    drone: Drone,
    tasks: Vec<DroneTask>,
    current_task_index: usize,
}

impl MissionDrone {
    pub fn new(mut drone: Drone, tasks: Vec<DroneTask>) -> Self {
        drone.assign_task(tasks.first().cloned());

        Self {
            drone,
            tasks,
            current_task_index: 0,
        }
    }

    pub fn drone(&self) -> &Drone {
        &self.drone
    }

    fn tick(&mut self, delta_time: f32) -> Result<(), AerisError> {
        self.drone.tick(delta_time)?;

        if self.drone.current_task().is_some() || self.is_finished() {
            return Ok(());
        }

        if self.current_task_index + 1 < self.tasks.len() {
            self.current_task_index += 1;
            self.drone
                .assign_task(self.tasks.get(self.current_task_index).cloned());
        }

        Ok(())
    }

    fn is_finished(&self) -> bool {
        !self.tasks.is_empty()
            && self.current_task_index + 1 == self.tasks.len()
            && self.drone.current_task().is_none()
            && self.drone.flight_mode() == &FlightMode::Idle
    }
}

#[derive(Debug)]
pub struct Mission {
    name: String,
    state: MissionState,
    drones: Vec<MissionDrone>,
}

impl Mission {
    pub fn new(name: String, drones: Vec<MissionDrone>) -> Self {
        Self {
            name,
            state: MissionState::Ready,
            drones,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> &MissionState {
        &self.state
    }

    pub fn drones(&self) -> impl Iterator<Item = &Drone> {
        self.drones.iter().map(MissionDrone::drone)
    }

    pub fn drone_count(&self) -> usize {
        self.drones.len()
    }

    pub fn start(&mut self) -> Result<(), AerisError> {
        match self.state {
            MissionState::Ready => {
                self.state = MissionState::Running;
                Ok(())
            }
            _ => Err(AerisError::InvalidMission(
                "mission can only be started from Ready state".to_string(),
            )),
        }
    }

    pub fn pause(&mut self) -> Result<(), AerisError> {
        match self.state {
            MissionState::Running => {
                self.state = MissionState::Paused;
                Ok(())
            }
            MissionState::Paused => Ok(()),
            _ => Err(AerisError::InvalidMission(
                "mission can only be paused while Running".to_string(),
            )),
        }
    }

    pub fn resume(&mut self) -> Result<(), AerisError> {
        match self.state {
            MissionState::Paused => {
                self.state = MissionState::Running;
                Ok(())
            }
            MissionState::Running => Ok(()),
            _ => Err(AerisError::InvalidMission(
                "mission can only be resumed from Paused state".to_string(),
            )),
        }
    }

    pub fn tick(&mut self, delta_time: f32) -> Result<(), AerisError> {
        if self.state != MissionState::Running {
            return Ok(());
        }

        for drone in &mut self.drones {
            drone.tick(delta_time)?;
        }

        if self.drones.iter().all(MissionDrone::is_finished) {
            self.state = MissionState::Finished;
        }

        Ok(())
    }

    pub fn is_finished(&self) -> bool {
        self.state == MissionState::Finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coordinates::Coordinates, drone::DroneConfig};

    fn mission_drone(climb_speed: f32, descent_speed: f32) -> MissionDrone {
        let mut drone = Drone::new(
            Coordinates::new(0.0, 0.0),
            0.0,
            0.0,
            DroneConfig {
                name: "test".to_string(),
                max_speed: 10.0,
                climb_speed,
                descent_speed,
                max_altitude: 100.0,
                battery_capacity: 100.0,
                consumption_per_second: 0.0,
            },
        );

        drone.connect().unwrap();
        drone.arm().unwrap();

        MissionDrone::new(
            drone,
            vec![
                DroneTask::Takeoff {
                    target_altitude: 1.0,
                },
                DroneTask::Land,
            ],
        )
    }

    #[test]
    fn mission_finishes_only_after_all_drones_are_idle() {
        let mut mission = Mission::new(
            "test".to_string(),
            vec![mission_drone(1.0, 1.0), mission_drone(0.5, 0.5)],
        );
        mission.start().unwrap();

        mission.tick(1.0).unwrap();
        mission.tick(1.0).unwrap();

        assert!(!mission.is_finished());

        mission.tick(1.0).unwrap();
        mission.tick(1.0).unwrap();

        assert!(mission.is_finished());
        assert!(
            mission
                .drones()
                .all(|drone| drone.flight_mode() == &FlightMode::Idle)
        );
    }

    #[test]
    fn paused_mission_does_not_advance_drones() {
        let mut mission = Mission::new("test".to_string(), vec![mission_drone(1.0, 1.0)]);
        mission.start().unwrap();
        mission.pause().unwrap();

        mission.tick(1.0).unwrap();

        assert_eq!(mission.drones().next().unwrap().altitude(), 0.0);
    }
}
