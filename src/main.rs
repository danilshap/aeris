use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{
    drone::MissionValidator,
    errors::AerisError,
    loader::{load_drone_catalog, load_mission_config},
    setup::build_simulation,
};

mod coordinates;
mod drone;
mod errors;
mod loader;
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

    let (mut simulation, mut drone_missions) = build_simulation(&mission_config, &drone_catalog)?;

    ratatui::run(|terminal| {
        let mut ui_state = ui::UiState::new();
        let mut last_tick = Instant::now();
        let mut mission_finished = false;

        while !mission_finished {
            terminal.draw(|frame| ui::draw(frame, &simulation, &mission_config, &mut ui_state))?;

            let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => ui_state.previous_drone(),
                    KeyCode::Down => ui_state.next_drone(simulation.drones().len()),
                    _ => {}
                }
            }

            if last_tick.elapsed() >= TICK_RATE {
                simulation.tick(DELTA_TIME)?;

                for (drone_id, mission) in &mut drone_missions {
                    simulation.update_mission(*drone_id, mission)?;
                }

                mission_finished = drone_missions
                    .iter()
                    .all(|(_, mission)| mission.is_finished());
                last_tick = Instant::now();
            }
        }

        terminal.draw(|frame| ui::draw(frame, &simulation, &mission_config, &mut ui_state))?;
        Ok(())
    })
}
