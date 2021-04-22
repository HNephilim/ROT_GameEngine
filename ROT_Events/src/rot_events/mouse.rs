use crate::rot_events::commom::State;


#[derive(Debug)]
pub struct MouseButton {
    pub state: State,
    pub button: Button,
}

#[derive(Debug)]
pub struct MouseWheel {
    pub line_delta: Coord<f64>,
}

#[derive(Debug)]
pub struct MouseMovement {
    pub position: Coord<f64>,
}

#[derive(Debug)]
pub enum Button {
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
