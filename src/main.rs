use crate::{
    drone::{Coordinates, Drone, DroneTask, Mission},
    errors::AerisError,
    loader::{load_drone_catalog, load_mission_config},
    simulation::Simulation,
};

mod drone;
mod errors;
mod loader;
mod mission_config;
mod simulation;

const DELTA_TIME: f32 = 0.1;

fn main() -> Result<(), AerisError> {
    let drone_catalog = load_drone_catalog("configs/drones.toml")?;
    let mission_config = load_mission_config("configs/mission.toml")?;

    let mut simulation = Simulation::new();

    for group in mission_config.groups {
        let drone_config = drone_catalog
            .drones
            .iter()
            .find(|config| config.name == group.drone_type)
            .ok_or_else(|| AerisError::DroneTypeNotFound(group.drone_type.clone()))?;

        for _ in 0..group.count {
            let coordinates = Coordinates {
                latitude: 50.4501,
                longitude: 30.5234,
            };

            let mut drone = Drone::new(coordinates, 0.0, 0.0, 100, drone_config.clone());

            drone.connect()?;
            drone.arm()?;

            if let Some(first_task) = group.tasks.first() {
                drone.assign_task(Some(first_task.clone()));
            }

            simulation.add_drone(drone);
        }
    }

    println!("Mission '{}' started", mission_config.name);

    for tick in 1..=200 {
        simulation.tick(DELTA_TIME)?;

        println!("Tick #{tick:03}");

        for drone in simulation.drones() {
            println!(
                "  {} | alt: {:>5.1}m | mode: {:?} | task: {:?}",
                drone.id(),
                drone.altitude(),
                drone.flight_mode(),
                drone.current_task(),
            );
        }
    }

    Ok(())
}
