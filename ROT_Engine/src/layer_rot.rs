use log::{debug, error, info, trace, warn};
use rot_events::ROT_MouseInput::ROT_MouseEvent;

#[derive(Clone)]
pub struct Layer {
    name: String,
    is_enabled: bool,
    actual_index: Option<usize>,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Layer {
            name,
            is_enabled: false,
            actual_index: None,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn enable(&mut self) -> &mut Self {
        self.is_enabled = true;
        self
    }

    pub fn disable(&mut self) -> &mut Self {
        self.is_enabled = false;
        self
    }

    pub fn get_index(&self) -> Option<usize> {
        self.actual_index
    }

    pub fn assign_index(&mut self, index: usize) {
        self.actual_index = Some(index)
    }

    pub fn disable_index(&mut self) {
        self.actual_index = None
    }

    pub fn on_event(&self, event: ROT_MouseEvent) -> bool {
        false
    }

    pub fn on_update(&mut self) {}
}
