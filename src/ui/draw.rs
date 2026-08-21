use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{
    coordinates::Coordinates, drone::DroneTask, mission_config::MissionConfig,
    simulation::Simulation, ui::UiState,
};

pub fn draw(
    frame: &mut Frame,
    simulation: &Simulation,
    mission_config: &MissionConfig,
    ui_state: &mut UiState,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let details_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[2]);

    draw_header(frame, layout[0], simulation, mission_config, ui_state);

    draw_fleet(frame, layout[1], simulation, ui_state);

    draw_drone_details(frame, details_layout[0], simulation, ui_state);

    draw_mission(frame, details_layout[1]);

    draw_logs(frame, layout[3]);

    draw_footer(frame, layout[4]);
}

// ---------------------------------------------------------
// Header
// ---------------------------------------------------------

fn draw_header(
    frame: &mut Frame,
    area: Rect,
    simulation: &Simulation,
    mission_config: &MissionConfig,
    ui_state: &UiState,
) {
    let selected = ui_state
        .selected_drone()
        .map(|index| format!("#{}", index + 1))
        .unwrap_or_else(|| "-".to_string());

    let text = format!(
        "Mission: {}    Status: RUNNING    Drones: {}    Selected: {}",
        mission_config.name,
        simulation.drones().len(),
        selected,
    );

    let widget =
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" AERIS "));

    frame.render_widget(widget, area);
}

fn draw_fleet(frame: &mut Frame, area: Rect, simulation: &Simulation, ui_state: &mut UiState) {
    let items = simulation
        .drones()
        .iter()
        .map(|drone| {
            let progress = match drone.current_task() {
                Some(DroneTask::FlyTo { target }) => calculate_flight_progress(
                    drone.flight_start_position(),
                    drone.coordinates(),
                    target,
                ),

                Some(DroneTask::ReturnHome) => calculate_flight_progress(
                    drone.flight_start_position(),
                    drone.coordinates(),
                    drone.home_position(),
                ),

                Some(DroneTask::Land) => 1.0,

                _ => 0.0,
            };

            let flight_bar = flight_progress_bar(progress, 30);

            let text = format!(
                "{:<8} {}   {:<12} Alt {:>5.1}m",
                drone.id(),
                flight_bar,
                format_task(drone.current_task()),
                drone.altitude(),
            );

            ListItem::new(text)
        })
        .collect::<Vec<_>>();

    let total = items.len();

    let selected = ui_state
        .selected_drone()
        .map(|index| index + 1)
        .unwrap_or(0);

    let title = format!(" FLEET OVERVIEW | {selected}/{total} ");

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut ui_state.fleet);
}

fn calculate_flight_progress(
    start: &Coordinates,
    current: &Coordinates,
    target: &Coordinates,
) -> f64 {
    let total_distance = distance(start, target);

    if total_distance <= f64::EPSILON {
        return 1.;
    }

    let remaining_distance = distance(current, target);

    let process = 1. - remaining_distance / total_distance;

    process.clamp(0., 1.)
}

fn distance(first: &Coordinates, second: &Coordinates) -> f64 {
    let dx = second.longitude() - first.longitude();
    let dy = second.latitude() - first.latitude();

    (dx * dx + dy * dy).sqrt()
}

fn flight_progress_bar(progress: f64, width: usize) -> String {
    let progress = progress.clamp(0., 1.);

    if width == 0 {
        return String::new();
    }

    let drone_position = (progress * (width - 1) as f64).round() as usize;

    let mut line = String::with_capacity(width + 4);

    line.push('●');
    line.push(' ');

    for index in 0..width {
        if index == drone_position {
            line.push('✈');
        } else {
            line.push('─');
        }
    }

    line.push(' ');
    line.push('○');

    line
}

fn format_task(task: Option<&DroneTask>) -> &'static str {
    match task {
        Some(DroneTask::Takeoff { .. }) => "TAKEOFF",
        Some(DroneTask::FlyTo { .. }) => "FLY TO",
        Some(DroneTask::ReturnHome) => "RETURN HOME",
        Some(DroneTask::Land) => "LAND",
        Some(DroneTask::Hold) => "HOLD",
        None => "IDLE",
    }
}

fn draw_drone_details(frame: &mut Frame, area: Rect, simulation: &Simulation, ui_state: &UiState) {
    let Some(index) = ui_state.selected_drone() else {
        draw_empty_drone(frame, area);
        return;
    };

    let Some(drone) = simulation.drones().get(index) else {
        draw_empty_drone(frame, area);
        return;
    };

    let text = format!(
        "\
Connection    {:?}
Flight mode   {:?}
Task          {:?}
Altitude      {:.1} m",
        drone.connection_status(),
        drone.flight_mode(),
        drone.current_task(),
        drone.altitude(),
    );

    let title = format!(" SELECTED DRONE — {} ", drone.id());

    let widget = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(widget, area);
}

fn draw_empty_drone(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new("No drone selected").block(
        Block::default()
            .borders(Borders::ALL)
            .title(" SELECTED DRONE "),
    );

    frame.render_widget(widget, area);
}

fn draw_mission(frame: &mut Frame, area: Rect) {

    let text = "\
Current step    FlyTo

✓ Takeoff
● FlyTo
○ Hold
○ ReturnHome
○ Land";

    let widget =
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" MISSION "));

    frame.render_widget(widget, area);
}

fn draw_logs(frame: &mut Frame, area: Rect) {

    let text = "\
00:01:42  DR-012  Flying to waypoint
00:01:41  DR-008  Reached altitude
00:01:39  DR-014  Taking off";

    let widget =
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" EVENT LOG "));

    frame.render_widget(widget, area);
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new(" ↑/↓ Select    R Return Home    Space Pause    Q Quit ")
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_progress_between_coordinates() {
        let start = Coordinates::new(10.0, 20.0);
        let current = Coordinates::new(15.0, 30.0);
        let target = Coordinates::new(20.0, 40.0);

        assert_eq!(calculate_flight_progress(&start, &current, &target), 0.5);
    }
}
