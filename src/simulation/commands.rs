use std::{
    sync::mpsc::{Receiver, RecvTimeoutError, SyncSender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::{
    drone::{DroneCommand, DroneEvent},
    mission::DroneHandle,
};

use super::{FleetSnapshot, Simulation};

pub enum SimulationCommand {
    Pause,
    Resume,
    Shutdown,
}

pub enum SimulationEvent {
    Snapshot(FleetSnapshot),
    Finished,
    Failed(String),
}

/// Runs simulation coordination and all mission drones on dedicated threads.
///
/// The simulation is consumed to give each drone a single owner. This worker sends
/// ticks, rebuilds the fleet snapshot from drone events, and joins every drone thread
/// before it exits. Pausing suppresses ticks without stopping the threads.
pub fn spawn_simulation_worker(
    simulation: Simulation,
    commands: Receiver<SimulationCommand>,
    events: SyncSender<SimulationEvent>,
    tick_rate: Duration,
    delta_time: f32,
) -> JoinHandle<()> {
    let mut fleet_snapshot = simulation.snapshot();
    let mission_drones = simulation.into_mission_drones();

    thread::spawn(move || {
        let drone_workers = mission_drones
            .into_iter()
            .map(DroneHandle::spawn)
            .collect::<Vec<_>>();

        let mut finished_drones = vec![false; drone_workers.len()];
        let mut paused = fleet_snapshot.paused;
        let mut next_tick = Instant::now() + tick_rate;

        loop {
            match commands.recv_timeout(next_tick.saturating_duration_since(Instant::now())) {
                Ok(SimulationCommand::Pause) => {
                    paused = true;
                    fleet_snapshot.paused = true;

                    if events
                        .send(SimulationEvent::Snapshot(fleet_snapshot.clone()))
                        .is_err()
                    {
                        break;
                    }

                    continue;
                }
                Ok(SimulationCommand::Resume) => {
                    paused = false;
                    fleet_snapshot.paused = false;

                    if events
                        .send(SimulationEvent::Snapshot(fleet_snapshot.clone()))
                        .is_err()
                    {
                        break;
                    }

                    continue;
                }
                Ok(SimulationCommand::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {}
            }

            next_tick = Instant::now() + tick_rate;

            if paused {
                continue;
            }

            if let Err(error) = tick_drones(
                &drone_workers,
                &mut fleet_snapshot,
                &mut finished_drones,
                delta_time,
            ) {
                let _ = events.send(SimulationEvent::Failed(error));
                break;
            }

            if events
                .send(SimulationEvent::Snapshot(fleet_snapshot.clone()))
                .is_err()
            {
                break;
            }

            if fleet_snapshot.finished {
                let _ = events.send(SimulationEvent::Finished);
                break;
            }
        }

        shutdown_drones(drone_workers, &finished_drones);
    })
}

fn tick_drones(
    workers: &[DroneHandle],
    snapshot: &mut FleetSnapshot,
    finished: &mut [bool],
    delta_time: f32,
) -> Result<(), String> {
    for (index, worker) in workers.iter().enumerate() {
        if finished[index] {
            continue;
        }

        worker
            .send(DroneCommand::Tick(delta_time))
            .map_err(|error| error.to_string())?;

        let (drone_snapshot, current_task_index, is_finished) = match worker.recv() {
            Ok(DroneEvent::Telemetry {
                snapshot,
                current_task_index,
            }) => (snapshot, current_task_index, false),
            Ok(DroneEvent::Finished {
                snapshot,
                current_task_index,
            }) => (snapshot, current_task_index, true),
            Ok(DroneEvent::Failed(error)) => return Err(error),
            Err(error) => return Err(error.to_string()),
        };

        snapshot.drones[index].drone = drone_snapshot;
        snapshot.drones[index].current_task_index = current_task_index;
        finished[index] = is_finished;
    }

    snapshot.finished = !finished.is_empty() && finished.iter().all(|finished| *finished);

    snapshot.progress = if snapshot.drones.is_empty() {
        0.0
    } else {
        snapshot
            .drones
            .iter()
            .enumerate()
            .map(|(index, drone)| {
                if finished[index] {
                    1.0
                } else if drone.tasks.is_empty() {
                    0.0
                } else {
                    drone.current_task_index as f64 / drone.tasks.len() as f64
                }
            })
            .sum::<f64>()
            / snapshot.drones.len() as f64
    };

    Ok(())
}

fn shutdown_drones(workers: Vec<DroneHandle>, finished: &[bool]) {
    for (index, worker) in workers.iter().enumerate() {
        if !finished[index] {
            let _ = worker.send(DroneCommand::Shutdown);
        }
    }

    for worker in workers {
        let _ = worker.join();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use crate::mission::Mission;

    use super::*;

    #[test]
    fn worker_handles_pause_resume_and_shutdown() {
        let mut mission = Mission::new("test".to_string(), vec![]);
        mission.start().unwrap();

        let mut simulation = Simulation::new();
        simulation.add_mission(mission);

        let (command_sender, command_receiver) = mpsc::sync_channel(2);
        let (event_sender, event_receiver) = mpsc::sync_channel(2);
        let worker = spawn_simulation_worker(
            simulation,
            command_receiver,
            event_sender,
            Duration::from_secs(60),
            0.1,
        );

        command_sender.send(SimulationCommand::Pause).unwrap();
        let SimulationEvent::Snapshot(snapshot) =
            event_receiver.recv_timeout(Duration::from_secs(1)).unwrap()
        else {
            panic!("expected paused simulation snapshot");
        };
        assert!(snapshot.paused);

        command_sender.send(SimulationCommand::Resume).unwrap();
        let SimulationEvent::Snapshot(snapshot) =
            event_receiver.recv_timeout(Duration::from_secs(1)).unwrap()
        else {
            panic!("expected resumed simulation snapshot");
        };
        assert!(!snapshot.paused);

        command_sender.send(SimulationCommand::Shutdown).unwrap();
        worker.join().unwrap();
    }

    #[test]
    fn worker_sends_snapshot_and_stops_on_shutdown() {
        let simulation = Simulation::new();
        let (command_sender, command_receiver) = mpsc::sync_channel(2);
        let (event_sender, event_receiver) = mpsc::sync_channel(2);
        let worker = spawn_simulation_worker(
            simulation,
            command_receiver,
            event_sender,
            Duration::from_millis(1),
            0.1,
        );

        let event = event_receiver.recv_timeout(Duration::from_secs(1)).unwrap();

        let SimulationEvent::Snapshot(snapshot) = event else {
            panic!("expected simulation snapshot");
        };

        assert_eq!(snapshot.drones.len(), 0);

        command_sender.send(SimulationCommand::Shutdown).unwrap();
        worker.join().unwrap();
    }
}
