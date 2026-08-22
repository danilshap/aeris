use std::sync::mpsc::{Receiver, Sender};

use crossterm::event::KeyCode;

use crate::{
    commands::{SimulationCommand, SimulationEvent},
    errors::AerisError,
    simulation::Simulation,
    ui::UiState,
};

pub struct App {
    simulation: Simulation,
    ui_state: UiState,
    commands: Sender<SimulationCommand>,
    events: Receiver<SimulationEvent>,
    should_quit: bool,
    failure: Option<String>,
}

impl App {
    pub fn new(
        simulation: Simulation,
        ui_state: UiState,
        commands: Sender<SimulationCommand>,
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

    pub fn simulation(&self) -> &Simulation {
        &self.simulation
    }

    pub fn mission_name(&self) -> Option<&str> {
        self.simulation.mission_name()
    }

    pub fn ui_state(&self) -> &UiState {
        &self.ui_state
    }

    pub fn fleet_state(&mut self) -> (&Simulation, &mut UiState) {
        (&self.simulation, &mut self.ui_state)
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn is_paused(&self) -> bool {
        self.simulation.is_paused()
    }

    pub fn is_finished(&self) -> bool {
        self.simulation.is_finished()
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
        self.ui_state.next_drone(self.simulation.drone_count());
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
