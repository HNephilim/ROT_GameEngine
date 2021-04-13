#[allow(unused_imports)]
#[allow(non_camel_case_types)]
use crate::rot_events::ROT_KeyboardInput::ROT_KeyboardInputEvent;
use crate::rot_events::ROT_MouseInput::ROT_MouseEvent;
use std::fmt::{Debug, Formatter, Pointer};

#[derive(Debug)]
pub enum ROT_Event {
    MouseInput(ROT_MouseEvent),
    KeyboardInput(ROT_KeyboardInputEvent),
}

#[derive(Debug, PartialEq)]
pub enum ROT_State {
    Pressed,
    Released,
}
