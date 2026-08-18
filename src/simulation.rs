use crate::{
    drone::{Drone, DroneTask},
    errors::AerisError,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct Simulation {
    drones: Vec<Drone>,
}

impl Simulation {
    pub fn new() -> Self {
        Self { drones: vec![] }
    }

    pub fn drones(&self) -> &Vec<Drone> {
        &self.drones
    }

    pub fn add_drone(&mut self, drone: Drone) {
        self.drones.push(drone);
    }

    pub fn assign_task(&mut self, drone_id: Uuid, task: DroneTask) -> Result<(), AerisError> {
        let drone = self
            .drones
            .iter_mut()
            .find(|drone| drone.id() == drone_id)
            .ok_or(AerisError::DroneNotFound)?;

        drone.assign_task(Some(task));

        Ok(())
    }

    pub fn tick(&mut self, delta_time: f32) -> Result<(), AerisError> {
        for drone in self.drones.iter_mut() {
            drone.tick(delta_time)?;
        }

        Ok(())
    }
}
