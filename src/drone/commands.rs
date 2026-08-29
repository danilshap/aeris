use super::DroneSnapshot;

pub enum DroneCommand {
    Tick(f32),
    Shutdown,
}

pub enum DroneEvent {
    Telemetry {
        sequence_number: u64,
        snapshot: DroneSnapshot,
        current_task_index: usize,
    },
    Finished {
        sequence_number: u64,
        snapshot: DroneSnapshot,
        current_task_index: usize,
    },
    Failed(String),
}
