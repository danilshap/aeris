use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use crate::{app::App, errors::AerisError, loader::load_drone_catalog};

mod app;
mod coordinates;
mod drone;
mod errors;
mod loader;
mod mission;
mod setup;
mod simulation;
mod ui;

const DELTA_TIME: f32 = 0.1;
const TICK_RATE: Duration = Duration::from_millis(330);

fn main() -> Result<(), AerisError> {
    let drone_catalog = load_drone_catalog("configs/drones.toml")?;
    let ui_state = ui::UiState::new();
    let mut app = App::new(drone_catalog, ui_state);

    let ui_result = ratatui::run(|terminal| {
        while !app.should_quit() {
            app.receive_simulation_events();

            terminal.draw(|frame| ui::draw(frame, &mut app))?;

            if event::poll(TICK_RATE)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                app.handle_key(key.code)?;
            }
        }

        terminal.draw(|frame| ui::draw(frame, &mut app))?;
        Ok(())
    });

    ui_result
}
