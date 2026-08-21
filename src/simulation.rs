use crate::{
    drone::Drone,
    errors::AerisError,
    mission::{Mission, MissionState},
};

#[derive(Debug)]
pub struct Simulation {
    missions: Vec<Mission>,
}

impl Simulation {
    pub fn new() -> Self {
        Self { missions: vec![] }
    }

    pub fn add_mission(&mut self, mission: Mission) {
        self.missions.push(mission);
    }

    pub fn drones(&self) -> impl Iterator<Item = &Drone> {
        self.missions.iter().flat_map(Mission::drones)
    }

    pub fn drone(&self, index: usize) -> Option<&Drone> {
        self.drones().nth(index)
    }

    pub fn drone_count(&self) -> usize {
        self.missions.iter().map(Mission::drone_count).sum()
    }

    pub fn mission_name(&self) -> Option<&str> {
        self.missions.first().map(Mission::name)
    }

    pub fn tick(&mut self, delta_time: f32) -> Result<(), AerisError> {
        for mission in &mut self.missions {
            mission.tick(delta_time)?;
        }

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), AerisError> {
        for mission in &mut self.missions {
            if !mission.is_finished() {
                mission.pause()?;
            }
        }

        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), AerisError> {
        for mission in &mut self.missions {
            if !mission.is_finished() {
                mission.resume()?;
            }
        }

        Ok(())
    }

    pub fn is_paused(&self) -> bool {
        self.missions
            .iter()
            .any(|mission| mission.state() == &MissionState::Paused)
    }

    pub fn is_finished(&self) -> bool {
        !self.missions.is_empty() && self.missions.iter().all(Mission::is_finished)
    }
}
