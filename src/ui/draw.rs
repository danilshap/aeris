use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use crate::{
    app::{App, MISSIONS},
    drone::{ConnectionStatus, DroneTask, FlightMode},
    simulation::FleetSnapshot,
    ui::UiState,
};

const ACCENT: Color = Color::LightCyan;
const HOT: Color = Color::LightMagenta;
const MUTED: Color = Color::DarkGray;

pub fn draw(frame: &mut Frame, app: &mut App) {
    if app.is_home() {
        draw_start_screen(frame, app);
        return;
    }

    let fleet_height = app.simulation().drones.len() as u16 + 3;
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

fn draw_start_screen(frame: &mut Frame, app: &App) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(frame.area());
    let center = |area| {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ])
            .split(area)[1]
    };

    let logo = [
        r"    _    _____ ____  ___ ____  ",
        r"   / \  | ____|  _ \|_ _/ ___| ",
        r"  / _ \ |  _| | |_) || |\___ \ ",
        r" / ___ \| |___|  _ < | | ___) |",
        r"/_/   \_\_____|_| \_\___|____/ ",
    ]
    .join("\n");
    frame.render_widget(
        Paragraph::new(logo)
            .alignment(Alignment::Center)
            .style(Style::new().fg(ACCENT).bold()),
        center(vertical[1]),
    );
    frame.render_widget(
        Paragraph::new("AUTONOMOUS DRONE MISSION SIMULATOR")
            .alignment(Alignment::Center)
            .style(MUTED),
        center(vertical[2]),
    );

    let items = MISSIONS
        .iter()
        .map(|mission| {
            ListItem::new(vec![
                Line::from(format!("  {}", mission.name)).fg(ACCENT).bold(),
                Line::from(format!("     {}", mission.description)).fg(Color::Gray),
            ])
        })
        .collect::<Vec<_>>();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(MUTED)
                .title(" SELECT MISSION ".fg(HOT)),
        )
        .highlight_symbol("▶ ")
        .highlight_style(Style::new().bg(Color::Rgb(12, 26, 34)).bold());
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected_mission()));
    frame.render_stateful_widget(list, center(vertical[3]), &mut state);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            " ↑/↓ ".fg(ACCENT).bold(),
            Span::raw("mission    "),
            "enter ".fg(Color::Green).bold(),
            Span::raw("start    "),
            "q ".fg(HOT).bold(),
            Span::raw("quit"),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(MUTED)),
        center(vertical[4]),
    );
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
        .and_then(|index| app.simulation().drones.get(index))
        .map(|mission_drone| mission_drone.drone.name.as_str())
        .unwrap_or("-");

    let text = Line::from(vec![
        " ◆ AERIS ".fg(HOT).bold(),
        Span::raw("  "),
        app.mission_name().unwrap_or("NO MISSION").fg(Color::Gray),
        Span::raw("  "),
        format!("▶ {status}").fg(status_color).bold(),
        Span::raw(format!(
            "    {} UNITS    SELECTED {selected}",
            app.simulation().drones.len()
        )),
    ]);

    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(MUTED)),
        area,
    );
}

fn draw_mission_progress(frame: &mut Frame, area: Rect, app: &App) {
    let progress = app.simulation().progress;
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

fn draw_fleet(frame: &mut Frame, area: Rect, simulation: &FleetSnapshot, ui_state: &mut UiState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(MUTED)
        .title(format!(" FLEET  {} UNITS ", simulation.drones.len()).fg(Color::Gray));
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
        .drones
        .iter()
        .map(|mission_drone| {
            let drone = &mission_drone.drone;
            let progress = drone.task_progress();
            let phase = if mission_drone.failure.is_some() {
                format!("{:<15}", "FAILED")
                    .fg(Color::Red)
                    .bold()
                    .slow_blink()
            } else {
                format!("{:<15}", format!("{:?}", drone.flight_mode))
                    .fg(phase_color(&drone.flight_mode))
            };
            let battery_color = if drone.battery_percentage < 20.0 {
                Color::Red
            } else if drone.battery_percentage < 40.0 {
                Color::Yellow
            } else {
                Color::Gray
            };

            ListItem::new(Line::from(vec![
                format!("{:<13}", drone.name).fg(Color::Gray).bold(),
                format!("{:<14}", format_task(drone.current_task.as_ref())).fg(ACCENT),
                phase,
                format!("{:>3.0}%  ", drone.battery_percentage).fg(battery_color),
                format!("{:<8}", signal_bar(&drone.connection_status)).fg(ACCENT),
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
    simulation: &FleetSnapshot,
    selected: Option<usize>,
) {
    let Some(mission_drone) = selected.and_then(|index| simulation.drones.get(index)) else {
        frame.render_widget(
            Paragraph::new("No drone selected")
                .block(Block::default().borders(Borders::ALL).border_style(MUTED)),
            area,
        );
        return;
    };

    let drone = &mission_drone.drone;
    let id = drone.id.to_string();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(MUTED)
        .title(format!(" SELECTED UNIT  {} · {} ", drone.name, &id[..8]).fg(Color::Gray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);
    let battery_color = if drone.battery_percentage < 20.0 {
        Color::Red
    } else if drone.battery_percentage < 40.0 {
        Color::Yellow
    } else {
        ACCENT
    };
    let telemetry = vec![
        Line::from(vec![" TELEMETRY".fg(ACCENT).bold()]),
        Line::from(format!(
            " alt {:>5.1} m    speed {:>5.1} m/s",
            drone.altitude, drone.speed
        )),
        Line::from(format!(
            " mode {:<12?} {:?}",
            drone.flight_mode, drone.connection_status
        )),
        Line::from(vec![
            Span::raw(" bat  "),
            progress_bar(drone.battery_percentage as f64 / 100.0, 8).fg(battery_color),
            format!(
                " {:>3.0}%   {}",
                drone.battery_percentage,
                signal_bar(&drone.connection_status)
            )
            .fg(Color::Gray),
        ]),
        Line::from(format!(
            " pos  {:.5}, {:.5}",
            drone.coordinates.latitude(),
            drone.coordinates.longitude()
        )),
    ];
    frame.render_widget(Paragraph::new(telemetry).style(Color::Gray), columns[0]);

    let current = mission_drone.current_task_index;
    let finished = drone.current_task.is_none() && drone.flight_mode == FlightMode::Idle;
    let progress = drone.task_progress();
    let mut tasks = vec![Line::from(
        format!(
            " TASK QUEUE  {}/{:02}",
            current + 1,
            mission_drone.tasks.len()
        )
        .fg(ACCENT)
        .bold(),
    )];
    tasks.extend(
        mission_drone
            .tasks
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
        "esc ".fg(Color::Green).bold(),
        Span::raw("menu    "),
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
    use crate::{
        coordinates::Coordinates,
        drone::{Drone, DroneConfig, DroneSnapshot},
    };

    #[test]
    fn calculates_landing_progress_from_starting_altitude() {
        let mut drone = Drone::new(
            "test".to_string(),
            Coordinates::new(0.0, 0.0),
            100.0,
            0.0,
            DroneConfig {
                name: "test".to_string(),
                max_speed: 10.0,
                climb_speed: 1.0,
                descent_speed: 1.0,
                max_altitude: 100.0,
                battery_capacity: 100.0,
                consumption_per_second: 0.0,
            },
        );
        drone.assign_task(Some(DroneTask::Land));
        drone.tick(25.0).unwrap();

        assert_eq!(DroneSnapshot::from(&drone).task_progress(), 0.25);
    }
}
