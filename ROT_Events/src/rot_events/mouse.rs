use crate::rot_events::commom::ROT_State;
use crate::rot_events::ROT_Event_Base::ROT_Event;

#[derive(Debug)]
pub struct ROT_MouseEvent {
    pub ev_type: TypeOfMouseEvent,
    pub mouse_button: Option<ROT_MouseButton>,
    pub mouse_wheel: Option<ROT_MouseWheel>,
    pub mouse_movement: Option<ROT_MouseMovement>,
}

#[derive(Debug)]
pub struct ROT_MouseButton {
    pub state: ROT_State,
    pub button: ROT_Button,
}

#[derive(Debug)]
pub struct ROT_MouseWheel {
    pub line_delta: Coord<f64>,
}

#[derive(Debug)]
pub struct ROT_MouseMovement {
    pub position: Coord<f64>,
}

#[derive(Debug)]
pub enum ROT_Button {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug)]
pub enum TypeOfMouseEvent {
    Button,
    Wheel,
    Movement,
}

#[derive(Debug)]
pub struct Coord<T> {
    pub x: T,
    pub y: T,
}
