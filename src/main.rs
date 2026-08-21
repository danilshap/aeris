use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyEventKind};

use crate::{
    app::App,
    errors::AerisError,
    loader::{load_drone_catalog, load_mission_config},
    mission::MissionValidator,
    setup::build_simulation,
};

mod app;
mod coordinates;
mod drone;
mod errors;
mod loader;
mod mission;
mod mission_config;
mod setup;
mod simulation;
mod ui;

const DELTA_TIME: f32 = 0.1;
const TICK_RATE: Duration = Duration::from_millis(330);

fn main() -> Result<(), AerisError> {
    let drone_catalog = load_drone_catalog("configs/drones.toml")?;
    let mission_config = load_mission_config("configs/mission.toml")?;

    MissionValidator::validate(&mission_config, &drone_catalog)?;

    let simulation = build_simulation(&mission_config, &drone_catalog)?;

    let ui_state = ui::UiState::new();

    let mut app = App::new(simulation, ui_state);

    ratatui::run(|terminal| {
        let mut last_tick = Instant::now();

        while !app.should_quit() {
            terminal.draw(|frame| ui::draw(frame, &mut app))?;

            let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                app.handle_key(key.code)?;
            }

            if last_tick.elapsed() >= TICK_RATE {
                app.tick(DELTA_TIME)?;
                last_tick = Instant::now();
            }
        }

        terminal.draw(|frame| ui::draw(frame, &mut app))?;
        Ok(())
    })
}
