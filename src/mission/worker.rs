use std::{
    sync::mpsc::{self, Receiver, SyncSender},
    thread::{self, JoinHandle},
};

use crate::drone::{DroneCommand, DroneEvent, DroneSnapshot};

use super::MissionDrone;

/// Owns the command channel, event channel, and thread of one mission drone.
///
/// Every accepted tick produces one event. The worker exits after reporting
/// completion or failure, or after receiving shutdown or losing its command channel.
pub struct DroneHandle {
    commands: SyncSender<DroneCommand>,
    events: Receiver<DroneEvent>,
    worker: JoinHandle<()>,
}

impl DroneHandle {
    pub fn spawn(mission_drone: MissionDrone) -> Self {
        let (command_sender, command_receiver) = mpsc::sync_channel(8);
        let (event_sender, event_receiver) = mpsc::sync_channel(8);

        let worker = spawn_drone_worker(mission_drone, command_receiver, event_sender);

        Self {
            commands: command_sender,
            events: event_receiver,
            worker,
        }
    }

    pub fn send(&self, command: DroneCommand) -> Result<(), mpsc::SendError<DroneCommand>> {
        self.commands.send(command)
    }

    pub fn recv(&self) -> Result<DroneEvent, mpsc::RecvError> {
        self.events.recv()
    }

    pub fn join(self) -> thread::Result<()> {
        self.worker.join()
    }
}

fn spawn_drone_worker(
    mut mission_drone: MissionDrone,
    commands: Receiver<DroneCommand>,
    events: SyncSender<DroneEvent>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(DroneCommand::Tick(delta_time)) = commands.recv() {
            if let Err(error) = mission_drone.tick(delta_time) {
                let _ = events.send(DroneEvent::Failed(error.to_string()));
                break;
            }

            let snapshot = DroneSnapshot::from(mission_drone.drone());
            let current_task_index = mission_drone.current_task_index();
            let finished = mission_drone.is_finished();

            let event = if finished {
                DroneEvent::Finished {
                    snapshot,
                    current_task_index,
                }
            } else {
                DroneEvent::Telemetry {
                    snapshot,
                    current_task_index,
                }
            };

            if events.send(event).is_err() {
                break;
            }

            if finished {
                break;
            }
        }
    })
}
