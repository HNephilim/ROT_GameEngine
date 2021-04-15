#[allow(unused_imports)]
#[allow(non_camel_case_types)]
use imgui::{BackendFlags, Context, Key};
use rot_events::ROT_Event_Base::{ROT_Event, ROT_State};
use rot_events::{
    ROT_KeyboardInput::{KeyCode, ROT_KeyboardInputEvent},
    ROT_MouseInput::{
        Coord, ROT_Button, ROT_MouseButton, ROT_MouseEvent, ROT_MouseMovement, ROT_MouseWheel,
        TypeOfMouseEvent,
    },
};
use rot_layer::ROT_Layer;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::window::Window;

pub struct ROT_Gui {
    context: Context,
    scale_factor: f64,

    //for ROT_Layer implementation
    name: String,
    is_enabled: bool,
    index_on_stack: Option<usize>,
}

impl ROT_Gui {
    pub fn build(name: String, window: &Window) -> Self {
        let mut context = Context::create();

        let io = context.io_mut();
        io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
        io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);

        let scale_factor = window.scale_factor();
        io.display_framebuffer_scale = [scale_factor as f32, scale_factor as f32];

        let logical_size: LogicalSize<f32> = window.inner_size().to_logical(scale_factor);
        io.display_size = [logical_size.width as f32, logical_size.height as f32];

        ROT_Gui {
            context,
            scale_factor,
            name,
            is_enabled: false,
            index_on_stack: None,
        }
    }

    pub fn attach_window(&mut self, _window: &Window) {}
}

impl ROT_Layer for ROT_Gui {
    fn enable(&mut self) {
        self.is_enabled = true;
    }

    fn disable(&mut self) {
        self.is_enabled = false;
    }

    fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    fn on_event(&mut self, event: &ROT_Event) {
        let io = self.context.io_mut();
        match event {
            ROT_Event::MouseInput(ev) => match ev.ev_type {
                TypeOfMouseEvent::Button => {
                    let button = ev.mouse_button.as_ref().unwrap();
                    let pressed = button.state == ROT_State::Pressed;
                    match button.button {
                        ROT_Button::Left => io.mouse_down[0] = pressed,
                        ROT_Button::Right => io.mouse_down[1] = pressed,
                        ROT_Button::Middle => io.mouse_down[2] = pressed,
                        ROT_Button::Other(a) => io.mouse_down[a as usize] = pressed,
                    }
                }
                TypeOfMouseEvent::Wheel => {
                    let wheel = ev.mouse_wheel.as_ref().unwrap();
                    io.mouse_wheel = wheel.line_delta.y as f32;
                    io.mouse_wheel = wheel.line_delta.x as f32
                }
                TypeOfMouseEvent::Movement => {
                    let movement = ev.mouse_movement.as_ref().unwrap();
                    let x = movement.position.x / self.scale_factor;
                    let y = movement.position.y / self.scale_factor;
                    io.mouse_pos = [x as f32, y as f32];
                }
            },
            ROT_Event::KeyboardInput(ev) => {
                let pressed = ev.state == ROT_State::Pressed;
                let key = *ev.virtual_keycode.as_ref().unwrap();
                io.keys_down[key as usize] = pressed;

                match key {
                    KeyCode::LShift | KeyCode::RShift => io.key_shift = pressed,
                    KeyCode::LControl | KeyCode::RControl => io.key_ctrl = pressed,
                    KeyCode::LAlt | KeyCode::RAlt => io.key_alt = pressed,
                    KeyCode::LWin | KeyCode::RWin => io.key_super = pressed,
                    _ => {}
                }
            }
        }
    }

    fn on_update(&mut self, delta_time: f64) {
        let ui = self.context.frame();

        let mut demo_window_open = true;
        ui.show_demo_window(&mut demo_window_open);

        let draw_data = ui.render();
    }

    fn assign_index(&mut self, index: usize) {
        self.index_on_stack = Some(index);
    }

    fn disable_index(&mut self) {
        self.index_on_stack = None;
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}
