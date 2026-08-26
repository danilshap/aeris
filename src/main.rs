use std::{sync::mpsc, time::Duration};

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
    let initial_snapshot = simulation.snapshot();

    let (command_sender, command_receiver) = mpsc::sync_channel(8);
    let (event_sender, event_receiver) = mpsc::sync_channel(8);

    let simulation_worker = simulation::spawn_simulation_worker(
        simulation,
        command_receiver,
        event_sender,
        TICK_RATE,
        DELTA_TIME,
    );

    let ui_state = ui::UiState::new();

    let mut app = App::new(initial_snapshot, ui_state, command_sender, event_receiver);

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

    drop(app);

    simulation_worker
        .join()
        .expect("simulation worker panicked");

    ui_result
}
