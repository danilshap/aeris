mod commands;
mod snapshot;

pub use commands::{SimulationCommand, SimulationEvent, spawn_simulation_worker};
pub use snapshot::FleetSnapshot;

use crate::mission::{Mission, MissionDrone};

#[derive(Debug, Clone)]
pub struct Simulation {
    missions: Vec<Mission>,
}

impl Simulation {
    pub fn new() -> Self {
        Self { missions: vec![] }
    }

    pub fn into_mission_drones(self) -> Vec<MissionDrone> {
        self.missions
            .into_iter()
            .flat_map(Mission::into_drones)
            .collect()
    }

    pub fn add_mission(&mut self, mission: Mission) {
        self.missions.push(mission);
    }
}
