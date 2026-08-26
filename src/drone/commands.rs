use super::DroneSnapshot;

pub enum DroneCommand {
    Tick(f32),
    Shutdown,
}

pub enum DroneEvent {
    Telemetry {
        snapshot: DroneSnapshot,
        current_task_index: usize,
    },
    Finished {
        snapshot: DroneSnapshot,
        current_task_index: usize,
    },
    Failed(String),
}
