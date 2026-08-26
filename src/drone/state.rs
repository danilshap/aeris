#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Lost,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FlightMode {
    Idle,
    Armed,
    Takeoff,
    Hold,
    Mission,
    ReturnToHome,
    Landing,
}
