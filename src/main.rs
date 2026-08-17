use crate::{
    drone::{Coordinates, Drone, DroneTask, Mission},
    errors::AerisError,
    loader::{load_app_config, load_drone_catalog},
    simulation::Simulation,
};

mod app_config;
mod drone;
mod errors;
mod loader;
mod simulation;

const DELTA_TIME: f32 = 0.1;

fn main() -> Result<(), AerisError> {
    let drone_catalog = load_drone_catalog("configs/drones.toml")?;
    let app_config = load_app_config("configs/app.toml")?;

    let mut simulation = Simulation::new();

    let mut mission_drone_id = None;

    for fleet_entry in app_config.fleet {
        let drone_config = drone_catalog
            .drones
            .iter()
            .find(|config| config.name == fleet_entry.drone_type)
            .ok_or_else(|| AerisError::DroneTypeNotFound(fleet_entry.drone_type.clone()))?;

        for _ in 0..fleet_entry.count {
            let coordinates = Coordinates {
                latitude: 50.4501,
                longitude: 30.5234,
            };

            let mut drone = Drone::new(coordinates, 0.0, 0.0, 100, drone_config.clone());

            drone.connect()?;
            drone.arm()?;

            if mission_drone_id.is_none() {
                mission_drone_id = Some(drone.id);
            }

            simulation.add_drone(drone);
        }
    }

    let drone_id = mission_drone_id.ok_or(AerisError::DroneNotFound)?;

    let mut mission = Mission::new(
        "takeoff-and-land".to_string(),
        vec![
            DroneTask::Takeoff {
                target_altitude: 10.0,
            },
            DroneTask::ReturnHome,
            DroneTask::Land,
        ],
    );

    if let Some(task) = mission.current_task() {
        simulation.assign_task(drone_id, task.clone())?;
    }

    for tick in 1..=200 {
        simulation.tick(DELTA_TIME)?;

        let drone = simulation
            .drones()
            .iter()
            .find(|drone| drone.id == drone_id)
            .ok_or(AerisError::DroneNotFound)?;

        println!(
            "Tick #{tick:03} | alt: {:>5.1}m | mode: {:?} | task: {:?}",
            drone.altitude, drone.flight_mode, drone.current_task,
        );

        if drone.current_task.is_none() {
            mission.next_task();

            if mission.is_finished() {
                println!("Mission '{}' finished", mission.name);
                break;
            }

            if let Some(task) = mission.current_task() {
                simulation.assign_task(drone_id, task.clone())?;
            }
        }
    }

    Ok(())
}
