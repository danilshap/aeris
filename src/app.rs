use std::{
    sync::mpsc::{self, Receiver, SyncSender},
    thread::JoinHandle,
};

use crossterm::event::KeyCode;

use crate::{
    DELTA_TIME, TICK_RATE,
    drone::DroneCatalog,
    errors::AerisError,
    loader::load_mission_config,
    mission::MissionValidator,
    setup::build_simulation,
    simulation,
    simulation::{FleetSnapshot, SimulationCommand, SimulationEvent},
    ui::UiState,
};

pub struct MissionOption {
    pub name: &'static str,
    pub description: &'static str,
    pub path: &'static str,
}

pub const MISSIONS: [MissionOption; 3] = [
    MissionOption {
        name: "Recon Alpha",
        description: "Standard reconnaissance route",
        path: "configs/mission.toml",
    },
    MissionOption {
        name: "Fault Injection",
        description: "One random drone fails during flight",
        path: "configs/mission_failure.toml",
    },
    MissionOption {
        name: "Long Patrol",
        description: "Five-minute endurance mission",
        path: "configs/mission_long.toml",
    },
];

struct SimulationSession {
    simulation: FleetSnapshot,
    commands: SyncSender<SimulationCommand>,
    events: Receiver<SimulationEvent>,
    worker: JoinHandle<()>,
}

pub struct App {
    drone_catalog: DroneCatalog,
    ui_state: UiState,
    selected_mission: usize,
    session: Option<SimulationSession>,
    should_quit: bool,
}

impl App {
    pub fn new(drone_catalog: DroneCatalog, ui_state: UiState) -> Self {
        Self {
            drone_catalog,
            ui_state,
            selected_mission: 0,
            session: None,
            should_quit: false,
        }
    }

    pub fn is_home(&self) -> bool {
        self.session.is_none()
    }

    pub fn selected_mission(&self) -> usize {
        self.selected_mission
    }

    pub fn simulation(&self) -> &FleetSnapshot {
        &self
            .session
            .as_ref()
            .expect("simulation is not running")
            .simulation
    }

    pub fn mission_name(&self) -> Option<&str> {
        self.simulation().mission_name.as_deref()
    }

    pub fn ui_state(&self) -> &UiState {
        &self.ui_state
    }

    pub fn fleet_state(&mut self) -> (&FleetSnapshot, &mut UiState) {
        let session = self.session.as_ref().expect("simulation is not running");
        (&session.simulation, &mut self.ui_state)
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn is_paused(&self) -> bool {
        self.simulation().paused
    }

    pub fn is_finished(&self) -> bool {
        self.simulation().finished
    }

    pub fn receive_simulation_events(&mut self) {
        let Some(session) = &mut self.session else {
            return;
        };

        while let Ok(event) = session.events.try_recv() {
            match event {
                SimulationEvent::Snapshot(simulation) => session.simulation = simulation,
                SimulationEvent::Finished => {}
            }
        }
    }

    pub fn previous_drone(&mut self) {
        self.ui_state.previous_drone();
    }

    pub fn next_drone(&mut self) {
        self.ui_state.next_drone(self.simulation().drones.len());
    }

    pub fn handle_key(&mut self, code: KeyCode) -> Result<(), AerisError> {
        if self.is_home() {
            match code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Enter => self.start_selected_mission()?,
                KeyCode::Up => self.selected_mission = self.selected_mission.saturating_sub(1),
                KeyCode::Down => {
                    self.selected_mission = (self.selected_mission + 1).min(MISSIONS.len() - 1)
                }
                _ => {}
            }

            return Ok(());
        }

        match code {
            KeyCode::Char('q') => {
                self.stop_mission();
                self.should_quit = true;
            }
            KeyCode::Esc => self.stop_mission(),
            KeyCode::Char(' ') if self.is_paused() => {
                let _ = self
                    .session
                    .as_ref()
                    .expect("simulation is not running")
                    .commands
                    .send(SimulationCommand::Resume);
            }
            KeyCode::Char(' ') => {
                let _ = self
                    .session
                    .as_ref()
                    .expect("simulation is not running")
                    .commands
                    .send(SimulationCommand::Pause);
            }
            KeyCode::Up => self.previous_drone(),
            KeyCode::Down => self.next_drone(),
            _ => {}
        }

        Ok(())
    }

    fn start_selected_mission(&mut self) -> Result<(), AerisError> {
        let mission_config = load_mission_config(MISSIONS[self.selected_mission].path)?;
        MissionValidator::validate(&mission_config, &self.drone_catalog)?;

        let simulation = build_simulation(&mission_config, &self.drone_catalog)?;
        let initial_snapshot = simulation.snapshot();
        let (command_sender, command_receiver) = mpsc::sync_channel(8);
        let (event_sender, event_receiver) = mpsc::sync_channel(8);
        let worker = simulation::spawn_simulation_worker(
            simulation,
            command_receiver,
            event_sender,
            TICK_RATE,
            DELTA_TIME,
        );

        self.ui_state = UiState::new();
        self.session = Some(SimulationSession {
            simulation: initial_snapshot,
            commands: command_sender,
            events: event_receiver,
            worker,
        });

        Ok(())
    }

    fn stop_mission(&mut self) {
        let Some(session) = self.session.take() else {
            return;
        };

        let _ = session.commands.send(SimulationCommand::Shutdown);
        session.worker.join().expect("simulation worker panicked");
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop_mission();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::load_drone_catalog;

    #[test]
    fn starts_each_mission_and_returns_home() {
        let drone_catalog = load_drone_catalog("configs/drones.toml").unwrap();
        let mut app = App::new(drone_catalog, UiState::new());

        for selected_mission in 0..MISSIONS.len() {
            app.selected_mission = selected_mission;
            app.handle_key(KeyCode::Enter).unwrap();
            assert!(!app.is_home());

            app.handle_key(KeyCode::Esc).unwrap();
            assert!(app.is_home());
        }
    }
}
