use crate::drone::{DroneSnapshot, DroneTask};

use super::MissionDrone;

#[derive(Debug, Clone)]
pub struct MissionDroneSnapshot {
    pub drone: DroneSnapshot,
    pub tasks: Vec<DroneTask>,
    pub sequence_number: u64,
    pub pending_ticks: u64,
    pub current_task_index: usize,
    pub failure: Option<String>,
}

impl From<&MissionDrone> for MissionDroneSnapshot {
    fn from(mission_drone: &MissionDrone) -> Self {
        Self {
            drone: DroneSnapshot::from(mission_drone.drone()),
            tasks: mission_drone.tasks().to_vec(),
            sequence_number: 0,
            pending_ticks: 0,
            current_task_index: mission_drone.current_task_index(),
            failure: None,
        }
    }
}
