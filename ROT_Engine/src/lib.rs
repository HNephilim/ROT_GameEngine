//! # ROT_GameEngine
//! this is where the magic happens

use rot_vk::ROT_Renderer;

mod log_rot;
use log::{debug, error, info, trace, warn};

use rot_layer;
use rot_layer::LayerStack;

use rot_events::{
    ROT_EventTranslator, ROT_Event_Base::ROT_Event, ROT_KeyboardInput::ROT_KeyboardInputEvent,
    ROT_MouseInput::ROT_MouseEvent,
};

use rot_gui::ROT_Gui;

use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::Thread;

use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};

/// This struct will define all the engine behavior.
/// Everything happens inside it.

#[allow(unused_imports)]
#[allow(non_camel_case_types)]
pub struct ROT_Engine {
    //Buffer and Stacks
    layer_stack: LayerStack,

    //Renderer
    renderer: ROT_Renderer,

    //Event
    event_receiver: Option<Receiver<Arc<ROT_Event>>>,
    event_buffer: Vec<ROT_Event>,
}

// public impl block
impl ROT_Engine {
    pub fn build(dimensions: [u32; 2]) -> Self {
        //log inicialization, enables the debug, error, info, trace, warn macros
        log_rot::setup_logger().unwrap();
        //engine inicialization

        let mut layer_stack = LayerStack::new();
        let renderer = ROT_Renderer::build(dimensions, true, 2);
        let gui = Box::new(ROT_Gui::build("GUI".to_string(), renderer.get_window()));
        layer_stack.push_indexed(gui, 0);

        let event_buffer = Vec::new();

        ROT_Engine {
            layer_stack,
            renderer,

            event_receiver: None,
            event_buffer,
        }
    }

    pub fn close(&mut self) {
        self.renderer.destroy();
    }

    pub fn run(&mut self) {
        self.renderer.run();
    }

    fn dispatch_events(&mut self) {
        for event in &self.event_buffer {
            for layer in self.layer_stack.stack() {
                debug! {"{:?}", event}
                layer.on_event(event);
            }
        }
        self.event_buffer.clear();
    }

    fn update(&mut self, delta_time: f64) {
        for layer in self.layer_stack.stack() {
            layer.on_update(delta_time)
        }
    }
}
