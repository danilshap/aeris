use std::sync::mpsc::{Receiver, SyncSender};

use crossterm::event::KeyCode;

use crate::{
    errors::AerisError,
    simulation::{FleetSnapshot, SimulationCommand, SimulationEvent},
    ui::UiState,
};

pub struct App {
    simulation: FleetSnapshot,
    ui_state: UiState,
    commands: SyncSender<SimulationCommand>,
    events: Receiver<SimulationEvent>,
    should_quit: bool,
    failure: Option<String>,
}

impl App {
    pub fn new(
        simulation: FleetSnapshot,
        ui_state: UiState,
        commands: SyncSender<SimulationCommand>,
        events: Receiver<SimulationEvent>,
    ) -> Self {
        Self {
            simulation,
            ui_state,
            commands,
            events,
            should_quit: false,
            failure: None,
        }
    }

    pub fn simulation(&self) -> &FleetSnapshot {
        &self.simulation
    }

    pub fn mission_name(&self) -> Option<&str> {
        self.simulation.mission_name.as_deref()
    }

    pub fn ui_state(&self) -> &UiState {
        &self.ui_state
    }

    pub fn fleet_state(&mut self) -> (&FleetSnapshot, &mut UiState) {
        (&self.simulation, &mut self.ui_state)
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn is_paused(&self) -> bool {
        self.simulation.paused
    }

    pub fn is_finished(&self) -> bool {
        self.simulation.finished
    }

    pub fn receive_simulation_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                SimulationEvent::Snapshot(simulation) => self.simulation = simulation,
                SimulationEvent::Finished => self.should_quit = true,
                SimulationEvent::Failed(error) => {
                    self.failure = Some(error);
                    self.should_quit = true;
                }
            }
        }
    }

    pub fn previous_drone(&mut self) {
        self.ui_state.previous_drone();
    }

    pub fn next_drone(&mut self) {
        self.ui_state.next_drone(self.simulation.drones.len());
    }

    pub fn handle_key(&mut self, code: KeyCode) -> Result<(), AerisError> {
        match code {
            KeyCode::Char('q') => {
                let _ = self.commands.send(SimulationCommand::Shutdown);
                self.should_quit = true;
            }
            KeyCode::Char(' ') if self.is_paused() => {
                let _ = self.commands.send(SimulationCommand::Resume);
            }
            KeyCode::Char(' ') => {
                let _ = self.commands.send(SimulationCommand::Pause);
            }
            KeyCode::Up => self.previous_drone(),
            KeyCode::Down => self.next_drone(),
            _ => {}
        }

        Ok(())
    }
}
