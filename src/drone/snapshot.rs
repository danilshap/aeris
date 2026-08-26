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
            altitude: drone.altitude(),
            speed: drone.speed(),
            battery_percentage: drone.battery_percentage(),
            connection_status: drone.connection_status().clone(),
            flight_mode: drone.flight_mode().clone(),
            current_task: drone.current_task().cloned(),
        }
    }
}
