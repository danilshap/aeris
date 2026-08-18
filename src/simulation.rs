use crate::{
    drone::{Drone, DroneTask, Mission},
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

    pub fn drones(&self) -> &[Drone] {
        &self.drones
    }

    pub fn add_drone(&mut self, drone: Drone) {
        self.drones.push(drone);
    }

    pub fn drone(&self, drone_id: Uuid) -> Option<&Drone> {
        return self.drones.iter().find(|drone| drone.id() == drone_id);
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

    pub fn update_mission(
        &mut self,
        drone_id: Uuid,
        mission: &mut Mission,
    ) -> Result<(), AerisError> {
        let task_finished = self
            .drone(drone_id)
            .ok_or(AerisError::DroneNotFound)?
            .current_task()
            .is_none();

        if !task_finished || mission.is_finished() {
            return Ok(());
        }

        mission.next_task();

        if let Some(task) = mission.current_task() {
            self.assign_task(drone_id, task.clone())?
        }

        Ok(())
    }
}
