use std::fmt::{Debug, Formatter, Pointer};

pub trait ROT_Event {}

#[derive(Debug)]
pub enum ROT_State {
    Pressed,
    Released,
}
