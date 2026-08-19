use crate::{
    drone::MissionValidator,
    errors::AerisError,
    loader::{load_drone_catalog, load_mission_config},
    setup::build_simulation,
};

mod coordinates;
mod drone;
mod errors;
mod loader;
mod mission_config;
mod setup;
mod simulation;

const DELTA_TIME: f32 = 0.1;

fn main() -> Result<(), AerisError> {
    let drone_catalog = load_drone_catalog("configs/drones.toml")?;
    let mission_config = load_mission_config("configs/mission.toml")?;

    MissionValidator::validate(&mission_config, &drone_catalog)?;

    let (mut simulation, mut drone_missions) = build_simulation(&mission_config, &drone_catalog)?;

    println!("Mission '{}' started", mission_config.name);

    for tick in 1..=200 {
        simulation.tick(DELTA_TIME)?;

        for (drone_id, mission) in &mut drone_missions {
            simulation.update_mission(*drone_id, mission)?;
        }

        println!("Tick #{tick:03}");

        for drone in simulation.drones() {
            println!(
                "  {} | connection: {:?} | alt: {:>5.1}m | mode: {:?} | task: {:?}",
                drone.id(),
                drone.connection_status(),
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
