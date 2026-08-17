use crate::{drone::Drone, errors::AerisError};

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

    pub fn tick(&mut self, delta_time: f32) -> Result<(), AerisError> {
        for drone in self.drones.iter_mut() {
            drone.tick(delta_time)?;
        }

        Ok(())
    }
}
