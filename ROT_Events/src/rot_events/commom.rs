#[allow(unused_imports)]
#[allow(non_camel_case_types)]
use crate::rot_events::KeyboardInput::KeyboardInputEvent;
use crate::rot_events::MouseInput::MouseEvent;
use std::fmt::{Debug, Formatter, Pointer};

#[derive(Debug)]
pub enum Event {
    MouseInput(MouseEvent),
    KeyboardInput(KeyboardInputEvent),
}

#[derive(Debug, PartialEq)]
pub enum State {
    Pressed,
    Released,
}
