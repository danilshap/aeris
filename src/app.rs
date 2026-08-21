use crossterm::event::KeyCode;

use crate::{errors::AerisError, simulation::Simulation, ui::UiState};

pub struct App {
    simulation: Simulation,
    ui_state: UiState,
    should_quit: bool,
}

impl App {
    pub fn new(simulation: Simulation, ui_state: UiState) -> Self {
        Self {
            simulation,
            ui_state,
            should_quit: false,
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

    pub fn tick(&mut self, delta_time: f32) -> Result<(), AerisError> {
        self.simulation.tick(delta_time)?;

        if self.simulation.is_finished() {
            self.should_quit = true;
        }

        Ok(())
    }

    pub fn previous_drone(&mut self) {
        self.ui_state.previous_drone();
    }

    pub fn next_drone(&mut self) {
        self.ui_state.next_drone(self.simulation.drone_count());
    }

    pub fn handle_key(&mut self, code: KeyCode) -> Result<(), AerisError> {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char(' ') if self.is_paused() => self.simulation.resume()?,
            KeyCode::Char(' ') => self.simulation.pause()?,
            KeyCode::Up => self.previous_drone(),
            KeyCode::Down => self.next_drone(),
            _ => {}
        }

        Ok(())
    }
}
