use crate::{
    drone::{Drone, DroneCatalog},
    errors::AerisError,
    mission::{Mission, MissionConfig, MissionDrone},
    simulation::Simulation,
};

pub fn build_simulation(
    mission_config: &MissionConfig,
    drone_catalog: &DroneCatalog,
) -> Result<Simulation, AerisError> {
    let mut simulation = Simulation::new();
    let mut mission_drones = Vec::new();

    for group in &mission_config.groups {
        let drone_config = drone_catalog
            .drones
            .iter()
            .find(|config| config.name == group.drone_type)
            .ok_or_else(|| AerisError::DroneTypeNotFound(group.drone_type.clone()))?;

        for name in &group.drone_names {
            let mut drone = Drone::new(
                name.clone(),
                group.home_position,
                0.0,
                0.0,
                drone_config.clone(),
            );

            drone.connect()?;
            drone.arm()?;

            mission_drones.push(MissionDrone::new(drone, group.tasks.clone()));
        }
    }

    let mut mission = Mission::new(mission_config.name.clone(), mission_drones);
    mission.start()?;
    simulation.add_mission(mission);

    Ok(simulation)
}
