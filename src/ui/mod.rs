mod draw;

use ratatui::widgets::ListState;

pub use draw::*;

pub struct UiState {
    fleet: ListState,
}

impl UiState {
    pub fn new() -> Self {
        let mut fleet = ListState::default();

        fleet.select(Some(0));

        Self { fleet }
    }

    pub fn selected_drone(&self) -> Option<usize> {
        self.fleet.selected()
    }

    pub fn next_drone(&mut self, drone_count: usize) {
        if drone_count == 0 {
            return;
        }

        let current = self.fleet.selected().unwrap_or(0);

        let next = if current + 1 < drone_count {
            current + 1
        } else {
            current
        };

        self.fleet.select(Some(next));
    }

    pub fn previous_drone(&mut self) {
        let current = self.fleet.selected().unwrap_or(0);
        let previous = current.saturating_sub(1);
        self.fleet.select(Some(previous));
    }
}
