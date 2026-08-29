use uuid::Uuid;

use crate::coordinates::Coordinates;

use super::{ConnectionStatus, Drone, DroneTask, FlightMode};

#[derive(Debug, Clone)]
pub struct DroneSnapshot {
    pub id: Uuid,
    pub name: String,
    pub coordinates: Coordinates,
    pub home_position: Coordinates,
    pub flight_start_position: Coordinates,
    pub flight_start_altitude: f32,
    pub altitude: f32,
    pub speed: f32,
    pub battery_percentage: f32,
    pub connection_status: ConnectionStatus,
    pub flight_mode: FlightMode,
    pub current_task: Option<DroneTask>,
}

impl From<&Drone> for DroneSnapshot {
    fn from(drone: &Drone) -> Self {
        Self {
            id: drone.id(),
            name: drone.name().to_owned(),
            coordinates: *drone.coordinates(),
            home_position: *drone.home_position(),
            flight_start_position: *drone.flight_start_position(),
            flight_start_altitude: drone.flight_start_altitude,
            altitude: drone.altitude(),
            speed: drone.speed(),
            battery_percentage: drone.battery_percentage(),
            connection_status: drone.connection_status().clone(),
            flight_mode: drone.flight_mode().clone(),
            current_task: drone.current_task().cloned(),
        }
    }
}

impl DroneSnapshot {
    pub fn task_progress(&self) -> f64 {
        match self.current_task.as_ref() {
            Some(DroneTask::Takeoff { target_altitude }) => {
                (self.altitude / *target_altitude).clamp(0.0, 1.0) as f64
            }
            Some(DroneTask::FlyTo { target }) => {
                calculate_flight_progress(&self.flight_start_position, &self.coordinates, target)
            }
            Some(DroneTask::ReturnHome) => calculate_flight_progress(
                &self.flight_start_position,
                &self.coordinates,
                &self.home_position,
            ),
            Some(DroneTask::Hold) => 1.0,
            Some(DroneTask::Land) if self.flight_start_altitude > f32::EPSILON => {
                (1.0 - self.altitude / self.flight_start_altitude).clamp(0.0, 1.0) as f64
            }
            Some(DroneTask::Land) => 1.0,
            None if self.flight_mode == FlightMode::Idle => 1.0,
            None => 0.0,
        }
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
