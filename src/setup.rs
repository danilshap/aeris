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

    if mission_config.random_failure && !mission_drones.is_empty() {
        let failed_index = mission_drones[0].drone().id().as_u128() as usize % mission_drones.len();
        mission_drones[failed_index].fail_after(10);
    }

    let mut mission = Mission::new(mission_config.name.clone(), mission_drones);
    mission.start()?;
    simulation.add_mission(mission);

    Ok(simulation)
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, time::Duration};

    use super::*;
    use crate::{
        loader::{load_drone_catalog, load_mission_config},
        simulation::{SimulationCommand, SimulationEvent, spawn_simulation_worker},
    };

    #[test]
    fn failure_mission_fails_one_random_drone() {
        let drone_catalog = load_drone_catalog("configs/drones.toml").unwrap();
        let mission_config = load_mission_config("configs/mission_failure.toml").unwrap();
        let simulation = build_simulation(&mission_config, &drone_catalog).unwrap();
        let (command_sender, command_receiver) = mpsc::sync_channel(2);
        let (event_sender, event_receiver) = mpsc::sync_channel(128);
        let worker = spawn_simulation_worker(
            simulation,
            command_receiver,
            event_sender,
            Duration::from_millis(1),
            0.1,
        );

        let failed_drones = loop {
            let SimulationEvent::Snapshot(snapshot) =
                event_receiver.recv_timeout(Duration::from_secs(1)).unwrap()
            else {
                panic!("mission finished before injected failure");
            };
            let failed_drones = snapshot
                .drones
                .iter()
                .filter(|drone| drone.failure.is_some())
                .count();

            if failed_drones > 0 {
                break failed_drones;
            }
        };

        command_sender.send(SimulationCommand::Shutdown).unwrap();
        worker.join().unwrap();

        assert_eq!(failed_drones, 1);
    }

    #[test]
    fn long_mission_takes_about_five_minutes() {
        let drone_catalog = load_drone_catalog("configs/drones.toml").unwrap();
        let mission_config = load_mission_config("configs/mission_long.toml").unwrap();
        let simulation = build_simulation(&mission_config, &drone_catalog).unwrap();
        let (_command_sender, command_receiver) = mpsc::sync_channel(2);
        let (event_sender, event_receiver) = mpsc::sync_channel(128);
        let worker = spawn_simulation_worker(
            simulation,
            command_receiver,
            event_sender,
            Duration::from_millis(1),
            0.1,
        );

        let ticks = loop {
            if let SimulationEvent::Snapshot(snapshot) =
                event_receiver.recv_timeout(Duration::from_secs(2)).unwrap()
                && snapshot.finished
            {
                break snapshot.drones[0].sequence_number;
            }
        };

        worker.join().unwrap();

        let duration = Duration::from_secs_f64(ticks as f64 * 0.33);
        assert!(duration >= Duration::from_secs(285));
        assert!(duration <= Duration::from_secs(315));
    }
}
