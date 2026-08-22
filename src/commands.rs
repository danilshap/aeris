use std::{
    sync::mpsc::{Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::simulation::Simulation;

pub enum SimulationCommand {
    Pause,
    Resume,
    Shutdown,
}

pub enum SimulationEvent {
    Snapshot(Simulation),
    Finished,
    Failed(String),
}

pub fn spawn_simulation_worker(
    mut simulation: Simulation,
    commands: Receiver<SimulationCommand>,
    events: Sender<SimulationEvent>,
    tick_rate: Duration,
    delta_time: f32,
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let result = match commands.recv_timeout(tick_rate) {
                Ok(SimulationCommand::Pause) => simulation.pause(),
                Ok(SimulationCommand::Resume) => simulation.resume(),
                Err(RecvTimeoutError::Timeout) => simulation.tick(delta_time),
                Err(RecvTimeoutError::Disconnected) | Ok(SimulationCommand::Shutdown) => break,
            };

            if let Err(error) = result {
                let _ = events.send(SimulationEvent::Failed(error.to_string()));
                break;
            }

            if events
                .send(SimulationEvent::Snapshot(simulation.clone()))
                .is_err()
            {
                break;
            }

            if simulation.is_finished() {
                let _ = events.send(SimulationEvent::Finished);
                break;
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn worker_sends_snapshot_and_stops_on_shutdown() {
        let simulation = Simulation::new();
        let (command_sender, command_receiver) = mpsc::channel();
        let (event_sender, event_receiver) = mpsc::channel();
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

        assert_eq!(snapshot.drone_count(), 0);

        command_sender.send(SimulationCommand::Shutdown).unwrap();
        worker.join().unwrap();
    }
}
