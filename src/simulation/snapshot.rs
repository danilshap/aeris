use crate::mission::{Mission, MissionDroneSnapshot};

use super::Simulation;

#[derive(Debug, Clone)]
pub struct FleetSnapshot {
    pub mission_name: Option<String>,
    pub paused: bool,
    pub finished: bool,
    pub progress: f64,
    pub drones: Vec<MissionDroneSnapshot>,
}

impl Simulation {
    pub fn snapshot(&self) -> FleetSnapshot {
        let progress = if self.missions.is_empty() {
            0.0
        } else {
            self.missions.iter().map(Mission::progress).sum::<f64>() / self.missions.len() as f64
        };

        FleetSnapshot {
            mission_name: self
                .missions
                .first()
                .map(|mission| mission.name().to_owned()),
            paused: false,
            finished: false,
            progress,
            drones: self
                .missions
                .iter()
                .flat_map(Mission::mission_drones)
                .map(MissionDroneSnapshot::from)
                .collect(),
        }
    }
}
