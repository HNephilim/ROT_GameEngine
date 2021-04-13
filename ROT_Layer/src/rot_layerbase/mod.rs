#[allow(unused_imports)]
#[allow(non_camel_case_types)]
use log::{debug, error, info, trace, warn};
use rot_events::ROT_Event_Base::ROT_Event;
use std::sync::Arc;

pub trait ROT_Layer {
    fn enable(&mut self);

    fn disable(&mut self);

    fn is_enabled(&self) -> bool;

    fn on_event(&mut self, event: &ROT_Event);

    fn on_update(&mut self, delta_time: f64);

    fn assign_index(&mut self, index: usize);

    fn disable_index(&mut self);

    fn get_name(&self) -> &String;
}
