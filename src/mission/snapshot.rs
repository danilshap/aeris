use crate::drone::{DroneSnapshot, DroneTask};

use super::MissionDrone;

#[derive(Debug, Clone)]
pub struct MissionDroneSnapshot {
    pub drone: DroneSnapshot,
    pub tasks: Vec<DroneTask>,
    pub current_task_index: usize,
}

impl From<&MissionDrone> for MissionDroneSnapshot {
    fn from(mission_drone: &MissionDrone) -> Self {
        Self {
            drone: DroneSnapshot::from(mission_drone.drone()),
            tasks: mission_drone.tasks().to_vec(),
            current_task_index: mission_drone.current_task_index(),
        }
    }
}
