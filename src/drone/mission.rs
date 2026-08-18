use serde::Deserialize;

use crate::{errors::AerisError, simulation::{self, Simulation}};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum DroneTask {
    Takeoff { target_altitude: f32 },
    Hold,
    ReturnHome,
    Land,
}

#[derive(Debug)]
pub struct Mission {
    pub name: String,
    pub tasks: Vec<DroneTask>,
    current_task_index: usize,
}

impl Mission {
    pub fn new(name: String, tasks: Vec<DroneTask>) -> Self {
        Self {
            name,
            tasks,
            current_task_index: 0,
        }
    }

    pub fn current_task(&self) -> Option<&DroneTask> {
        self.tasks.get(self.current_task_index)
    }

    pub fn next_task(&mut self) {
        self.current_task_index += 1
    }

    pub fn is_finished(&self) -> bool {
        self.current_task().is_none()
    }
}
