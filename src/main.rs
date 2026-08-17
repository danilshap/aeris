use crate::{
    drone::{Coordinates, Drone, DroneTask},
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

            drone.assign_task(Some(DroneTask::Takeoff {
                target_altitude: 10.0,
            }));

            simulation.add_drone(drone);
        }
    }

    for tick in 1..=60 {
        simulation.tick(DELTA_TIME)?;

        println!("Tick #{tick}");

        for drone in simulation.drones() {
            println!(
                "  {} | alt: {:.1}m | mode: {:?} | task: {:?}",
                drone.id, drone.altitude, drone.flight_mode, drone.current_task,
            );
        }

        println!();
    }

    Ok(())
}
