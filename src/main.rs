use crate::{
    drone::{Coordinates, Drone, Mission},
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
    let mut drone_missions = Vec::new();

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

            let mut drone = Drone::new(coordinates, 0.0, 0.0, 100., drone_config.clone());

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

    println!("Mission '{}' started", mission_config.name);

    for tick in 1..=200 {
        simulation.tick(DELTA_TIME)?;

        for (drone_id, mission) in &mut drone_missions {
            simulation.update_mission(*drone_id, mission)?;
        }

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

        if drone_missions
            .iter()
            .all(|(_, mission)| mission.is_finished())
        {
            println!("Mission '{}' finished", mission_config.name);
            break;
        }
    }

    Ok(())
}
