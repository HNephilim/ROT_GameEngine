#[allow(unused_imports)]
#[allow(non_camel_case_types)]
use crate::ROT_KeyboardInput::{KeyCode, ROT_KeyboardInputEvent};
use crate::ROT_MouseInput::{
    Coord, ROT_Button, ROT_MouseButton, ROT_MouseEvent, ROT_MouseMovement, ROT_MouseWheel,
    TypeOfMouseEvent,
};

use crate::ROT_Event_Base::ROT_State;
use log::{debug, error, info, trace, warn};

use crate::rot_events::keyboard::KeyCode::{
    F1, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F2, F20, F21, F22, F23, F24, F3, F4, F5,
    F6, F7, F8, F9,
};
use winit::event::VirtualKeyCode::Escape;
use winit::event::WindowEvent::{CursorMoved, MouseInput, MouseWheel};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};

pub fn mouse_event(event: WindowEvent) -> Option<ROT_MouseEvent> {
    match event {
        WindowEvent::MouseInput { state, button, .. } => {
            let rot_state = match state {
                ElementState::Pressed => ROT_State::Pressed,
                ElementState::Released => ROT_State::Released,
            };

            let rot_button = match button {
                MouseButton::Left => ROT_Button::Left,
                MouseButton::Right => ROT_Button::Right,
                MouseButton::Middle => ROT_Button::Middle,
                MouseButton::Other(a) => ROT_Button::Other(a),
            };

            Some(ROT_MouseEvent {
                ev_type: TypeOfMouseEvent::Button,
                mouse_button: Some(ROT_MouseButton {
                    state: rot_state,
                    button: rot_button,
                }),
                mouse_wheel: None,
                mouse_movement: None,
            })
        }
        WindowEvent::CursorMoved { position, .. } => Some(ROT_MouseEvent {
            ev_type: TypeOfMouseEvent::Movement,
            mouse_button: None,
            mouse_wheel: None,
            mouse_movement: Some(ROT_MouseMovement {
                position: Coord {
                    x: position.x,
                    y: position.y,
                },
            }),
        }),
        WindowEvent::MouseWheel { delta, .. } => {
            let (x, y) = match delta {
                MouseScrollDelta::LineDelta(x, y) => (x as f64, y as f64),
                MouseScrollDelta::PixelDelta(pos) => (pos.x, pos.y),
            };
            Some(ROT_MouseEvent {
                ev_type: TypeOfMouseEvent::Wheel,
                mouse_button: None,
                mouse_wheel: Some(ROT_MouseWheel {
                    line_delta: Coord { x, y },
                }),
                mouse_movement: None,
            })
        }
        _ => None,
    }
}

pub fn keyboard_input_event(event: WindowEvent) -> Option<ROT_KeyboardInputEvent> {
    match event {
        WindowEvent::KeyboardInput { input, .. } => {
            let rot_state = match input.state {
                ElementState::Pressed => ROT_State::Pressed,
                ElementState::Released => ROT_State::Released,
            };

            let rot_scancode = input.scancode;

            let rot_key = match input.virtual_keycode {
                None => KeyCode::None,
                Some(key) => match key {
                    VirtualKeyCode::Key1 => KeyCode::Key1,
                    VirtualKeyCode::Key2 => KeyCode::Key2,
                    VirtualKeyCode::Key3 => KeyCode::Key3,
                    VirtualKeyCode::Key4 => KeyCode::Key4,
                    VirtualKeyCode::Key5 => KeyCode::Key5,
                    VirtualKeyCode::Key6 => KeyCode::Key6,
                    VirtualKeyCode::Key7 => KeyCode::Key7,
                    VirtualKeyCode::Key8 => KeyCode::Key8,
                    VirtualKeyCode::Key9 => KeyCode::Key9,
                    VirtualKeyCode::Key0 => KeyCode::Key0,
                    VirtualKeyCode::A => KeyCode::A,
                    VirtualKeyCode::B => KeyCode::B,
                    VirtualKeyCode::C => KeyCode::C,
                    VirtualKeyCode::D => KeyCode::D,
                    VirtualKeyCode::E => KeyCode::E,
                    VirtualKeyCode::F => KeyCode::F,
                    VirtualKeyCode::G => KeyCode::G,
                    VirtualKeyCode::H => KeyCode::H,
                    VirtualKeyCode::I => KeyCode::I,
                    VirtualKeyCode::J => KeyCode::J,
                    VirtualKeyCode::K => KeyCode::K,
                    VirtualKeyCode::L => KeyCode::L,
                    VirtualKeyCode::M => KeyCode::M,
                    VirtualKeyCode::N => KeyCode::N,
                    VirtualKeyCode::O => KeyCode::O,
                    VirtualKeyCode::P => KeyCode::P,
                    VirtualKeyCode::Q => KeyCode::Q,
                    VirtualKeyCode::R => KeyCode::R,
                    VirtualKeyCode::S => KeyCode::S,
                    VirtualKeyCode::T => KeyCode::T,
                    VirtualKeyCode::U => KeyCode::U,
                    VirtualKeyCode::V => KeyCode::V,
                    VirtualKeyCode::W => KeyCode::W,
                    VirtualKeyCode::X => KeyCode::X,
                    VirtualKeyCode::Y => KeyCode::Y,
                    VirtualKeyCode::Z => KeyCode::Z,
                    VirtualKeyCode::Escape => KeyCode::Esc,
                    VirtualKeyCode::F1 => KeyCode::F1,
                    VirtualKeyCode::F2 => KeyCode::F2,
                    VirtualKeyCode::F3 => KeyCode::F3,
                    VirtualKeyCode::F4 => KeyCode::F4,
                    VirtualKeyCode::F5 => KeyCode::F5,
                    VirtualKeyCode::F6 => KeyCode::F6,
                    VirtualKeyCode::F7 => KeyCode::F7,
                    VirtualKeyCode::F8 => KeyCode::F8,
                    VirtualKeyCode::F9 => KeyCode::F9,
                    VirtualKeyCode::F10 => KeyCode::F10,
                    VirtualKeyCode::F11 => KeyCode::F11,
                    VirtualKeyCode::F12 => KeyCode::F12,
                    VirtualKeyCode::F13 => KeyCode::F13,
                    VirtualKeyCode::F14 => KeyCode::F14,
                    VirtualKeyCode::F15 => KeyCode::F15,
                    VirtualKeyCode::F16 => KeyCode::F16,
                    VirtualKeyCode::F17 => KeyCode::F17,
                    VirtualKeyCode::F18 => KeyCode::F18,
                    VirtualKeyCode::F19 => KeyCode::F19,
                    VirtualKeyCode::F20 => KeyCode::F20,
                    VirtualKeyCode::F21 => KeyCode::F21,
                    VirtualKeyCode::F22 => KeyCode::F22,
                    VirtualKeyCode::F23 => KeyCode::F23,
                    VirtualKeyCode::F24 => KeyCode::F24,
                    VirtualKeyCode::Snapshot => KeyCode::Snapshot,
                    VirtualKeyCode::Scroll => KeyCode::Scroll,
                    VirtualKeyCode::Pause => KeyCode::Pause,
                    VirtualKeyCode::Insert => KeyCode::Insert,
                    VirtualKeyCode::Home => KeyCode::Home,
                    VirtualKeyCode::Delete => KeyCode::Delete,
                    VirtualKeyCode::End => KeyCode::End,
                    VirtualKeyCode::PageDown => KeyCode::PageDown,
                    VirtualKeyCode::PageUp => KeyCode::PageUp,
                    VirtualKeyCode::Left => KeyCode::Left,
                    VirtualKeyCode::Up => KeyCode::Up,
                    VirtualKeyCode::Right => KeyCode::Right,
                    VirtualKeyCode::Down => KeyCode::Down,
                    VirtualKeyCode::Back => KeyCode::Backspace,
                    VirtualKeyCode::Return => KeyCode::Return,
                    VirtualKeyCode::Space => KeyCode::Space,
                    VirtualKeyCode::Compose => KeyCode::Compose,
                    VirtualKeyCode::Caret => KeyCode::Caret,
                    VirtualKeyCode::Numlock => KeyCode::Numlock,
                    VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
                    VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
                    VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
                    VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
                    VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
                    VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
                    VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
                    VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
                    VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
                    VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
                    VirtualKeyCode::NumpadAdd => KeyCode::NumpadAdd,
                    VirtualKeyCode::NumpadDivide => KeyCode::NumpadDivide,
                    VirtualKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
                    VirtualKeyCode::NumpadComma => KeyCode::NumpadComma,
                    VirtualKeyCode::NumpadEnter => KeyCode::NumpadEnter,
                    VirtualKeyCode::NumpadEquals => KeyCode::NumpadEquals,
                    VirtualKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
                    VirtualKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
                    VirtualKeyCode::AbntC1 => KeyCode::AbntC1,
                    VirtualKeyCode::AbntC2 => KeyCode::AbntC2,
                    VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
                    VirtualKeyCode::Apps => KeyCode::Apps,
                    VirtualKeyCode::Asterisk => KeyCode::Asterisk,
                    VirtualKeyCode::At => KeyCode::At,
                    VirtualKeyCode::Ax => KeyCode::Ax,
                    VirtualKeyCode::Backslash => KeyCode::Backslash,
                    VirtualKeyCode::Calculator => KeyCode::Calculator,
                    VirtualKeyCode::Capital => KeyCode::Capital,
                    VirtualKeyCode::Colon => KeyCode::Colon,
                    VirtualKeyCode::Comma => KeyCode::Comma,
                    VirtualKeyCode::Convert => KeyCode::Convert,
                    VirtualKeyCode::Equals => KeyCode::Equals,
                    VirtualKeyCode::Grave => KeyCode::Grave,
                    VirtualKeyCode::Kana => KeyCode::Kana,
                    VirtualKeyCode::Kanji => KeyCode::Kanji,
                    VirtualKeyCode::LAlt => KeyCode::LAlt,
                    VirtualKeyCode::LBracket => KeyCode::LBracket,
                    VirtualKeyCode::LControl => KeyCode::LControl,
                    VirtualKeyCode::LShift => KeyCode::LShift,
                    VirtualKeyCode::LWin => KeyCode::LWin,
                    VirtualKeyCode::Mail => KeyCode::Mail,
                    VirtualKeyCode::MediaSelect => KeyCode::MediaSelect,
                    VirtualKeyCode::MediaStop => KeyCode::MediaStop,
                    VirtualKeyCode::Minus => KeyCode::Minus,
                    VirtualKeyCode::Mute => KeyCode::Mute,
                    VirtualKeyCode::MyComputer => KeyCode::MyComputer,
                    VirtualKeyCode::NavigateForward => KeyCode::NavigateForward,
                    VirtualKeyCode::NavigateBackward => KeyCode::NavigateBackward,
                    VirtualKeyCode::NextTrack => KeyCode::NextTrack,
                    VirtualKeyCode::NoConvert => KeyCode::NoConvert,
                    VirtualKeyCode::OEM102 => KeyCode::OEM102,
                    VirtualKeyCode::Period => KeyCode::Period,
                    VirtualKeyCode::PlayPause => KeyCode::PlayPause,
                    VirtualKeyCode::Plus => KeyCode::Plus,
                    VirtualKeyCode::Power => KeyCode::Power,
                    VirtualKeyCode::PrevTrack => KeyCode::PrevTrack,
                    VirtualKeyCode::RAlt => KeyCode::RAlt,
                    VirtualKeyCode::RBracket => KeyCode::RBracket,
                    VirtualKeyCode::RControl => KeyCode::RControl,
                    VirtualKeyCode::RShift => KeyCode::RShift,
                    VirtualKeyCode::RWin => KeyCode::RWin,
                    VirtualKeyCode::Semicolon => KeyCode::Semicolon,
                    VirtualKeyCode::Slash => KeyCode::Slash,
                    VirtualKeyCode::Sleep => KeyCode::Sleep,
                    VirtualKeyCode::Stop => KeyCode::Stop,
                    VirtualKeyCode::Sysrq => KeyCode::Sysrq,
                    VirtualKeyCode::Tab => KeyCode::Tab,
                    VirtualKeyCode::Underline => KeyCode::Underline,
                    VirtualKeyCode::Unlabeled => KeyCode::Unlabeled,
                    VirtualKeyCode::VolumeDown => KeyCode::VolumeDown,
                    VirtualKeyCode::VolumeUp => KeyCode::VolumeUp,
                    VirtualKeyCode::Wake => KeyCode::Wake,
                    VirtualKeyCode::WebBack => KeyCode::WebBack,
                    VirtualKeyCode::WebFavorites => KeyCode::WebFavorites,
                    VirtualKeyCode::WebForward => KeyCode::WebForward,
                    VirtualKeyCode::WebHome => KeyCode::WebHome,
                    VirtualKeyCode::WebRefresh => KeyCode::WebRefresh,
                    VirtualKeyCode::WebSearch => KeyCode::WebSearch,
                    VirtualKeyCode::WebStop => KeyCode::WebStop,
                    VirtualKeyCode::Yen => KeyCode::Yen,
                    VirtualKeyCode::Copy => KeyCode::Copy,
                    VirtualKeyCode::Paste => KeyCode::Paste,
                    VirtualKeyCode::Cut => KeyCode::Cut,
                },
            };

            Some(ROT_KeyboardInputEvent {
                state: rot_state,
                scancode: rot_scancode,
                virtual_keycode: Some(rot_key),
            })
        }
        _ => None,
    }
}
