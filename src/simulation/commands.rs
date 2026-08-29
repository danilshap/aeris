use std::{
    sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TryRecvError, TrySendError},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::{
    drone::{DroneCommand, DroneEvent, DroneSnapshot},
    mission::{DroneHandle, MissionDroneSnapshot},
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

                    if !try_send_snapshot(&events, &fleet_snapshot) {
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

            tick_drones(
                &drone_workers,
                &mut fleet_snapshot,
                &mut finished_drones,
                delta_time,
            );

            if fleet_snapshot.finished {
                if events
                    .send(SimulationEvent::Snapshot(fleet_snapshot.clone()))
                    .is_err()
                {
                    break;
                }

                let _ = events.send(SimulationEvent::Finished);
                break;
            }

            if !try_send_snapshot(&events, &fleet_snapshot) {
                break;
            }
        }

        shutdown_drones(drone_workers, &finished_drones);
    })
}

fn try_send_snapshot(events: &SyncSender<SimulationEvent>, snapshot: &FleetSnapshot) -> bool {
    !matches!(
        events.try_send(SimulationEvent::Snapshot(snapshot.clone())),
        Err(TrySendError::Disconnected(_))
    )
}

fn tick_drones(
    workers: &[DroneHandle],
    snapshot: &mut FleetSnapshot,
    finished: &mut [bool],
    delta_time: f32,
) {
    for (index, worker) in workers.iter().enumerate() {
        if finished[index] {
            continue;
        }

        let current_drone = &mut snapshot.drones[index];
        if current_drone.failure.is_some() {
            continue;
        }

        let (sequence_number, drone_snapshot, current_task_index, is_finished) =
            match worker.try_recv() {
                Ok(DroneEvent::Telemetry {
                    sequence_number,
                    snapshot,
                    current_task_index,
                }) => (sequence_number, snapshot, current_task_index, false),
                Ok(DroneEvent::Finished {
                    sequence_number,
                    snapshot,
                    current_task_index,
                }) => (sequence_number, snapshot, current_task_index, true),
                Ok(DroneEvent::Failed(error)) => {
                    current_drone.failure = Some(error);
                    continue;
                }
                Err(TryRecvError::Empty) => continue,
                Err(TryRecvError::Disconnected) => {
                    current_drone.failure = Some("drone worker disconnected".to_string());
                    continue;
                }
            };

        apply_drone_snapshot(
            current_drone,
            &mut finished[index],
            sequence_number,
            drone_snapshot,
            current_task_index,
            is_finished,
        );
    }

    for (index, worker) in workers.iter().enumerate() {
        if finished[index] {
            continue;
        }

        if snapshot.drones[index].failure.is_some() {
            continue;
        }

        schedule_drone_tick(&mut snapshot.drones[index], delta_time, |command| {
            worker.try_send(command)
        });
    }

    snapshot.finished = !finished.is_empty()
        && finished
            .iter()
            .enumerate()
            .all(|(index, finished)| *finished || snapshot.drones[index].failure.is_some());

    // Include active task progress so fleet progress updates between task transitions.
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
                    (drone.current_task_index as f64 + drone.drone.task_progress())
                        / drone.tasks.len() as f64
                }
            })
            .sum::<f64>()
            / snapshot.drones.len() as f64
    };
}

fn schedule_drone_tick(
    current_drone: &mut MissionDroneSnapshot,
    delta_time: f32,
    mut send: impl FnMut(DroneCommand) -> Result<(), TrySendError<DroneCommand>>,
) {
    current_drone.pending_ticks += 1;

    while current_drone.pending_ticks > 0 {
        match send(DroneCommand::Tick(delta_time)) {
            Ok(()) => current_drone.pending_ticks -= 1,
            Err(TrySendError::Full(_)) => break,
            Err(TrySendError::Disconnected(_)) => {
                current_drone.failure = Some("drone worker disconnected".to_string());
                break;
            }
        }
    }
}

fn apply_drone_snapshot(
    current_drone: &mut MissionDroneSnapshot,
    finished: &mut bool,
    sequence_number: u64,
    drone_snapshot: DroneSnapshot,
    current_task_index: usize,
    is_finished: bool,
) {
    if sequence_number <= current_drone.sequence_number {
        return;
    }

    current_drone.sequence_number = sequence_number;
    current_drone.drone = drone_snapshot;
    current_drone.current_task_index = current_task_index;
    *finished = is_finished;
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

    use crate::{
        coordinates::Coordinates,
        drone::{Drone, DroneConfig, DroneTask},
        mission::{Mission, MissionDrone},
    };

    use super::*;

    fn mission_drone_snapshot() -> MissionDroneSnapshot {
        let mission_drone = MissionDrone::new(
            Drone::new(
                "test".to_string(),
                Coordinates::new(0.0, 0.0),
                0.0,
                0.0,
                DroneConfig {
                    name: "test".to_string(),
                    max_speed: 10.0,
                    climb_speed: 1.0,
                    descent_speed: 1.0,
                    max_altitude: 100.0,
                    battery_capacity: 100.0,
                    consumption_per_second: 0.0,
                },
            ),
            vec![],
        );

        MissionDroneSnapshot::from(&mission_drone)
    }

    #[test]
    fn stale_snapshot_does_not_replace_newer_snapshot() {
        let mut current_drone = mission_drone_snapshot();
        let mut newer_snapshot = current_drone.drone.clone();
        newer_snapshot.altitude = 2.0;
        let mut finished = false;

        apply_drone_snapshot(
            &mut current_drone,
            &mut finished,
            2,
            newer_snapshot,
            2,
            true,
        );

        let mut stale_snapshot = current_drone.drone.clone();
        stale_snapshot.altitude = 1.0;
        apply_drone_snapshot(
            &mut current_drone,
            &mut finished,
            1,
            stale_snapshot,
            1,
            false,
        );

        assert_eq!(current_drone.sequence_number, 2);
        assert_eq!(current_drone.drone.altitude, 2.0);
        assert_eq!(current_drone.current_task_index, 2);
        assert!(finished);
    }

    #[test]
    fn fleet_progress_includes_current_task_progress() {
        let task = DroneTask::Takeoff {
            target_altitude: 100.0,
        };
        let mut drone = mission_drone_snapshot();
        drone.tasks = vec![task.clone(), DroneTask::Land];
        drone.drone.current_task = Some(task);
        drone.drone.altitude = 25.0;
        let mut snapshot = FleetSnapshot {
            mission_name: None,
            paused: false,
            finished: false,
            progress: 0.0,
            drones: vec![drone],
        };

        tick_drones(&[], &mut snapshot, &mut [false], 0.1);

        assert_eq!(snapshot.progress, 0.125);
    }

    #[test]
    fn pending_ticks_are_sent_when_channel_capacity_recovers() {
        let mut current_drone = mission_drone_snapshot();
        let (sender, receiver) = mpsc::sync_channel(2);

        for _ in 0..3 {
            schedule_drone_tick(&mut current_drone, 0.1, |command| sender.try_send(command));
        }

        assert_eq!(current_drone.pending_ticks, 1);

        receiver.recv().unwrap();
        receiver.recv().unwrap();

        schedule_drone_tick(&mut current_drone, 0.1, |command| sender.try_send(command));

        assert_eq!(current_drone.pending_ticks, 0);

        for _ in 0..2 {
            let DroneCommand::Tick(delta_time) = receiver.recv().unwrap() else {
                panic!("expected tick command");
            };
            assert_eq!(delta_time, 0.1);
        }
    }

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

    #[test]
    fn slow_event_consumer_does_not_block_simulation_worker() {
        let simulation = Simulation::new();
        let (command_sender, command_receiver) = mpsc::sync_channel(2);
        let (event_sender, event_receiver) = mpsc::sync_channel(1);
        let worker = spawn_simulation_worker(
            simulation,
            command_receiver,
            event_sender,
            Duration::from_millis(1),
            0.1,
        );

        event_receiver.recv_timeout(Duration::from_secs(1)).unwrap();
        thread::sleep(Duration::from_millis(20));
        command_sender.send(SimulationCommand::Shutdown).unwrap();

        let (stopped_sender, stopped_receiver) = mpsc::channel();
        thread::spawn(move || stopped_sender.send(worker.join()).unwrap());

        assert!(
            stopped_receiver
                .recv_timeout(Duration::from_millis(100))
                .unwrap()
                .is_ok()
        );
    }

    #[test]
    fn final_snapshot_is_delivered_when_event_channel_is_full() {
        let mut drone = Drone::new(
            "test".to_string(),
            Coordinates::new(0.0, 0.0),
            0.0,
            0.0,
            DroneConfig {
                name: "test".to_string(),
                max_speed: 10.0,
                climb_speed: 1.0,
                descent_speed: 1.0,
                max_altitude: 100.0,
                battery_capacity: 100.0,
                consumption_per_second: 0.0,
            },
        );
        drone.connect().unwrap();
        drone.arm().unwrap();

        let mut mission = Mission::new(
            "test".to_string(),
            vec![MissionDrone::new(drone, vec![DroneTask::Land])],
        );
        mission.start().unwrap();

        let mut simulation = Simulation::new();
        simulation.add_mission(mission);
        let (_command_sender, command_receiver) = mpsc::sync_channel(2);
        let (event_sender, event_receiver) = mpsc::sync_channel(1);
        let worker = spawn_simulation_worker(
            simulation,
            command_receiver,
            event_sender,
            Duration::from_millis(1),
            0.1,
        );

        thread::sleep(Duration::from_millis(20));
        event_receiver.recv_timeout(Duration::from_secs(1)).unwrap();

        let SimulationEvent::Snapshot(snapshot) =
            event_receiver.recv_timeout(Duration::from_secs(1)).unwrap()
        else {
            panic!("expected final snapshot before finished event");
        };
        assert!(snapshot.finished);
        assert!(matches!(
            event_receiver.recv_timeout(Duration::from_secs(1)),
            Ok(SimulationEvent::Finished)
        ));

        worker.join().unwrap();
    }

    #[test]
    fn failed_drone_does_not_stop_healthy_drone() {
        let config = DroneConfig {
            name: "test".to_string(),
            max_speed: 10.0,
            climb_speed: 1.0,
            descent_speed: 1.0,
            max_altitude: 100.0,
            battery_capacity: 100.0,
            consumption_per_second: 0.0,
        };
        let mut failed_drone = Drone::new(
            "failed".to_string(),
            Coordinates::new(0.0, 0.0),
            0.0,
            0.0,
            config.clone(),
        );
        failed_drone.connect().unwrap();
        failed_drone.arm().unwrap();
        let mut healthy_drone = Drone::new(
            "healthy".to_string(),
            Coordinates::new(0.0, 0.0),
            0.0,
            0.0,
            config,
        );
        healthy_drone.connect().unwrap();
        healthy_drone.arm().unwrap();

        let mut mission = Mission::new(
            "test".to_string(),
            vec![
                MissionDrone::new(
                    failed_drone,
                    vec![DroneTask::FlyTo {
                        target: Coordinates::new(0.0, 0.001),
                    }],
                ),
                MissionDrone::new(
                    healthy_drone,
                    vec![DroneTask::Takeoff {
                        target_altitude: 100.0,
                    }],
                ),
            ],
        );
        mission.start().unwrap();

        let mut simulation = Simulation::new();
        simulation.add_mission(mission);
        let (command_sender, command_receiver) = mpsc::sync_channel(2);
        let (event_sender, event_receiver) = mpsc::sync_channel(128);
        let worker = spawn_simulation_worker(
            simulation,
            command_receiver,
            event_sender,
            Duration::from_millis(1),
            0.1,
        );

        let mut healthy_altitude_after_failure = None;
        let mut healthy_drone_continued = false;

        for _ in 0..100 {
            let SimulationEvent::Snapshot(snapshot) =
                event_receiver.recv_timeout(Duration::from_secs(1)).unwrap()
            else {
                panic!("simulation finished before healthy drone continued");
            };

            if snapshot.drones[0].failure.is_none() {
                continue;
            }

            if let Some(altitude) = healthy_altitude_after_failure {
                if snapshot.drones[1].drone.altitude > altitude {
                    healthy_drone_continued = true;
                    break;
                }
            } else {
                healthy_altitude_after_failure = Some(snapshot.drones[1].drone.altitude);
            }
        }

        command_sender.send(SimulationCommand::Shutdown).unwrap();
        worker.join().unwrap();

        assert!(healthy_altitude_after_failure.is_some());
        assert!(healthy_drone_continued);
    }
}
