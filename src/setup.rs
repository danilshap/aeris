use crate::{
    drone::{Coordinates, Drone, Mission},
    errors::AerisError,
    mission_config::{DroneCatalog, MissionConfig},
    simulation::Simulation,
};
use uuid::Uuid;

const BASE_LATITUDE: f64 = 50.4501;
const BASE_LONGITUDE: f64 = 30.5234;

pub fn build_simulation(
    mission_config: &MissionConfig,
    drone_catalog: &DroneCatalog,
) -> Result<(Simulation, Vec<(Uuid, Mission)>), AerisError> {
    let mut simulation = Simulation::new();
    let mut drone_missions = Vec::new();

    for group in &mission_config.groups {
        let drone_config = drone_catalog
            .drones
            .iter()
            .find(|config| config.name == group.drone_type)
            .ok_or_else(|| AerisError::DroneTypeNotFound(group.drone_type.clone()))?;

        for _ in 0..group.count {
            let coordinates = Coordinates {
                latitude: BASE_LATITUDE,
                longitude: BASE_LONGITUDE,
            };

            let mut drone = Drone::new(coordinates, 0.0, 0.0, 100.0, drone_config.clone());

            drone.connect()?;
            drone.arm()?;

            let mission = Mission::new(mission_config.name.clone(), group.tasks.clone());

            if let Some(task) = mission.current_task() {
                drone.assign_task(Some(task.clone()));
            }

            let drone_id = drone.id();
            simulation.add_drone(drone);
            drone_missions.push((drone_id, mission));
        }
    }

    Ok((simulation, drone_missions))
}
