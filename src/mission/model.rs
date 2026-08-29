use crate::{
    drone::{Drone, DroneTask, FlightMode},
    errors::AerisError,
};

#[derive(Debug, Clone)]
pub struct MissionDrone {
    drone: Drone,
    tasks: Vec<DroneTask>,
    current_task_index: usize,
    tick_count: u64,
    failure_tick: Option<u64>,
}

impl MissionDrone {
    pub fn new(mut drone: Drone, tasks: Vec<DroneTask>) -> Self {
        drone.assign_task(tasks.first().cloned());

        Self {
            drone,
            tasks,
            current_task_index: 0,
            tick_count: 0,
            failure_tick: None,
        }
    }

    pub fn drone(&self) -> &Drone {
        &self.drone
    }

    pub fn tasks(&self) -> &[DroneTask] {
        &self.tasks
    }

    pub fn current_task_index(&self) -> usize {
        self.current_task_index
    }

    pub fn fail_after(&mut self, ticks: u64) {
        self.failure_tick = Some(ticks);
    }

    pub(super) fn tick(&mut self, delta_time: f32) -> Result<(), AerisError> {
        self.tick_count += 1;

        if self.failure_tick == Some(self.tick_count) {
            return Err(AerisError::SimulatedDroneFailure(
                self.drone.name().to_string(),
            ));
        }

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

    pub(super) fn is_finished(&self) -> bool {
        !self.tasks.is_empty()
            && self.current_task_index + 1 == self.tasks.len()
            && self.drone.current_task().is_none()
            && self.drone.flight_mode() == &FlightMode::Idle
    }

    fn progress(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }

        if self.is_finished() {
            return 1.0;
        }

        self.current_task_index as f64 / self.tasks.len() as f64
    }
}

#[derive(Debug, Clone)]
pub struct Mission {
    name: String,
    drones: Vec<MissionDrone>,
}

impl Mission {
    pub fn new(name: String, drones: Vec<MissionDrone>) -> Self {
        Self { name, drones }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn into_drones(self) -> Vec<MissionDrone> {
        self.drones
    }

    pub fn mission_drones(&self) -> impl Iterator<Item = &MissionDrone> {
        self.drones.iter()
    }

    pub fn progress(&self) -> f64 {
        if self.drones.is_empty() {
            return 0.0;
        }

        self.drones.iter().map(MissionDrone::progress).sum::<f64>() / self.drones.len() as f64
    }
}
