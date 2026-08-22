use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use crate::{
    app::App,
    coordinates::Coordinates,
    drone::{ConnectionStatus, Drone, FlightMode},
    mission::DroneTask,
    simulation::Simulation,
    ui::UiState,
};

const ACCENT: Color = Color::LightCyan;
const HOT: Color = Color::LightMagenta;
const MUTED: Color = Color::DarkGray;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let fleet_height = app.simulation().drone_count() as u16 + 3;
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(fleet_height),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_header(frame, layout[0], app);
    draw_mission_progress(frame, layout[1], app);

    let (simulation, ui_state) = app.fleet_state();
    draw_fleet(frame, layout[2], simulation, ui_state);

    draw_selected_drone(
        frame,
        layout[3],
        app.simulation(),
        app.ui_state().selected_drone(),
    );
    draw_footer(frame, layout[4], app.is_paused());
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let (status, status_color) = if app.is_finished() {
        ("FINISHED", ACCENT)
    } else if app.is_paused() {
        ("PAUSED", Color::Yellow)
    } else {
        ("RUNNING", Color::Green)
    };

    let selected = app
        .ui_state()
        .selected_drone()
        .and_then(|index| app.simulation().drone(index))
        .map(Drone::name)
        .unwrap_or("-");

    let text = Line::from(vec![
        " ◆ AERIS ".fg(HOT).bold(),
        Span::raw("  "),
        app.mission_name().unwrap_or("NO MISSION").fg(Color::Gray),
        Span::raw("  "),
        format!("▶ {status}").fg(status_color).bold(),
        Span::raw(format!(
            "    {} UNITS    SELECTED {selected}",
            app.simulation().drone_count()
        )),
    ]);

    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(MUTED)),
        area,
    );
}

fn draw_mission_progress(frame: &mut Frame, area: Rect, app: &App) {
    let progress = app.simulation().mission_progress();
    let label = format!("{:>3.0}%", progress * 100.0);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(MUTED)
                .title(" MISSION PROGRESS ".fg(ACCENT)),
        )
        .gauge_style(Style::new().fg(ACCENT).bg(Color::Black).bold())
        .ratio(progress)
        .label(label);

    frame.render_widget(gauge, area);
}

fn draw_fleet(frame: &mut Frame, area: Rect, simulation: &Simulation, ui_state: &mut UiState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(MUTED)
        .title(format!(" FLEET  {} UNITS ", simulation.drone_count()).fg(Color::Gray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    let header = Line::from(vec![
        Span::raw("  UNIT         "),
        "TASK          ".fg(ACCENT).bold(),
        Span::raw("PHASE          BAT   SIG      PROGRESS"),
    ]);
    frame.render_widget(Paragraph::new(header).style(MUTED), rows[0]);

    let items = simulation
        .drones()
        .map(|drone| {
            let progress = task_progress(drone);
            let phase_color = phase_color(drone.flight_mode());
            let battery_color = if drone.battery_percentage() < 20.0 {
                Color::Red
            } else if drone.battery_percentage() < 40.0 {
                Color::Yellow
            } else {
                Color::Gray
            };

            ListItem::new(Line::from(vec![
                format!("{:<13}", drone.name()).fg(Color::Gray).bold(),
                format!("{:<14}", format_task(drone.current_task())).fg(ACCENT),
                format!("{:<15}", format!("{:?}", drone.flight_mode())).fg(phase_color),
                format!("{:>3.0}%  ", drone.battery_percentage()).fg(battery_color),
                format!("{:<8}", signal_bar(drone.connection_status())).fg(ACCENT),
                progress_bar(progress, 10).fg(ACCENT),
                format!(" {:>3.0}%", progress * 100.0).fg(Color::Gray),
            ]))
        })
        .collect::<Vec<_>>();

    let list = List::new(items)
        .highlight_symbol("▶ ")
        .highlight_style(Style::new().bg(Color::Rgb(12, 26, 34)).bold());

    frame.render_stateful_widget(list, rows[1], &mut ui_state.fleet);
}

fn task_progress(drone: &Drone) -> f64 {
    match drone.current_task() {
        Some(DroneTask::Takeoff { target_altitude }) => {
            (drone.altitude() / target_altitude).clamp(0.0, 1.0) as f64
        }
        Some(DroneTask::FlyTo { target }) => {
            calculate_flight_progress(drone.flight_start_position(), drone.coordinates(), target)
        }
        Some(DroneTask::ReturnHome) => calculate_flight_progress(
            drone.flight_start_position(),
            drone.coordinates(),
            drone.home_position(),
        ),
        Some(DroneTask::Hold) => 1.0,
        Some(DroneTask::Land) => 0.0,
        None if drone.flight_mode() == &FlightMode::Idle => 1.0,
        None => 0.0,
    }
}

fn calculate_flight_progress(
    start: &Coordinates,
    current: &Coordinates,
    target: &Coordinates,
) -> f64 {
    let total_distance = distance(start, target);

    if total_distance <= f64::EPSILON {
        return 1.0;
    }

    (1.0 - distance(current, target) / total_distance).clamp(0.0, 1.0)
}

fn distance(first: &Coordinates, second: &Coordinates) -> f64 {
    let dx = second.longitude() - first.longitude();
    let dy = second.latitude() - first.latitude();

    (dx * dx + dy * dy).sqrt()
}

fn progress_bar(progress: f64, width: usize) -> String {
    let filled = (progress.clamp(0.0, 1.0) * width as f64).round() as usize;
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

fn signal_bar(status: &ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Connected => "▁▃▅▇",
        ConnectionStatus::Connecting => "▁▃▅░",
        ConnectionStatus::Disconnected => "▁░░░",
        ConnectionStatus::Lost => "░░░░",
    }
}

fn phase_color(mode: &FlightMode) -> Color {
    match mode {
        FlightMode::Emergency => Color::Red,
        FlightMode::Landing | FlightMode::ReturnToHome => Color::Yellow,
        FlightMode::Idle => MUTED,
        _ => ACCENT,
    }
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

fn draw_selected_drone(
    frame: &mut Frame,
    area: Rect,
    simulation: &Simulation,
    selected: Option<usize>,
) {
    let Some(mission_drone) = selected.and_then(|index| simulation.mission_drone(index)) else {
        frame.render_widget(
            Paragraph::new("No drone selected")
                .block(Block::default().borders(Borders::ALL).border_style(MUTED)),
            area,
        );
        return;
    };

    let drone = mission_drone.drone();
    let id = drone.id().to_string();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(MUTED)
        .title(format!(" SELECTED UNIT  {} · {} ", drone.name(), &id[..8]).fg(Color::Gray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);
    let battery_color = if drone.battery_percentage() < 20.0 {
        Color::Red
    } else if drone.battery_percentage() < 40.0 {
        Color::Yellow
    } else {
        ACCENT
    };
    let telemetry = vec![
        Line::from(vec![" TELEMETRY".fg(ACCENT).bold()]),
        Line::from(format!(
            " alt {:>5.1} m    speed {:>5.1} m/s",
            drone.altitude(),
            drone.speed()
        )),
        Line::from(format!(
            " mode {:<12?} {:?}",
            drone.flight_mode(),
            drone.connection_status()
        )),
        Line::from(vec![
            Span::raw(" bat  "),
            progress_bar(drone.battery_percentage() as f64 / 100.0, 8).fg(battery_color),
            format!(
                " {:>3.0}%   {}",
                drone.battery_percentage(),
                signal_bar(drone.connection_status())
            )
            .fg(Color::Gray),
        ]),
        Line::from(format!(
            " pos  {:.5}, {:.5}",
            drone.coordinates().latitude(),
            drone.coordinates().longitude()
        )),
    ];
    frame.render_widget(Paragraph::new(telemetry).style(Color::Gray), columns[0]);

    let current = mission_drone.current_task_index();
    let finished = drone.current_task().is_none() && drone.flight_mode() == &FlightMode::Idle;
    let progress = task_progress(drone);
    let mut tasks = vec![Line::from(
        format!(
            " TASK QUEUE  {}/{:02}",
            current + 1,
            mission_drone.tasks().len()
        )
        .fg(ACCENT)
        .bold(),
    )];
    tasks.extend(
        mission_drone
            .tasks()
            .iter()
            .enumerate()
            .skip(current.saturating_sub(1))
            .take(4)
            .map(|(index, task)| {
                let completed = index < current || finished && index == current;
                let active = index == current && !finished;
                let (marker, color) = if completed {
                    ("✓", Color::Green)
                } else if active {
                    ("▶", HOT)
                } else {
                    ("○", ACCENT)
                };
                let progress = if active {
                    format!("  {:>3.0}%", progress * 100.0)
                } else {
                    String::new()
                };

                Line::from(vec![
                    format!(" {marker} {:02} ", index + 1).fg(color).bold(),
                    format_task_detail(task).fg(Color::Gray),
                    progress.fg(color),
                ])
            }),
    );
    frame.render_widget(Paragraph::new(tasks), columns[1]);
}

fn format_task_detail(task: &DroneTask) -> String {
    match task {
        DroneTask::Takeoff { target_altitude } => {
            format!("takeoff → {target_altitude:.0} m")
        }
        DroneTask::FlyTo { target } => {
            format!(
                "fly to {:.4} / {:.4}",
                target.latitude(),
                target.longitude()
            )
        }
        DroneTask::ReturnHome => "return home".to_string(),
        DroneTask::Land => "land".to_string(),
        DroneTask::Hold => "hold".to_string(),
    }
}

fn draw_footer(frame: &mut Frame, area: Rect, paused: bool) {
    let pause_action = if paused { "resume" } else { "pause" };
    let text = Line::from(vec![
        " ↑/↓ ".fg(ACCENT).bold(),
        Span::raw("unit    "),
        "space ".fg(Color::Yellow).bold(),
        Span::raw(format!("{pause_action}    ")),
        "q ".fg(HOT).bold(),
        Span::raw("quit"),
    ]);

    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(MUTED)),
        area,
    );
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
