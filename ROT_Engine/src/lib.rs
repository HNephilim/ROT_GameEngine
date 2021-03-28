//! # ROT_GameEngine
//! this is where the magic happens
use rot_vk::ROT_Renderer;

mod log_rot;
use log::{debug, error, info, trace, warn};

mod layer_rot;
use layer_rot::Layer;

mod layerstack_rot;
use layerstack_rot::LayerStack;

use rot_events::{
    ROT_Event_Base::ROT_Event, ROT_KeyboardInput::ROT_KeyboardInputEvent,
    ROT_MouseInput::ROT_MouseEvent,
};

use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::Thread;

/// This struct will define all the engine behavior.
/// Everything happens inside it.

pub struct RotEngine {
    //Buffer and Stacks
    layer_stack: LayerStack,

    //Renderer
    renderer: ROT_Renderer,

    //Window
    event_receiver: Option<Receiver<Box<dyn ROT_Event + Send>>>,
}

// public impl block
impl RotEngine {
    pub fn build() -> Self {
        //log inicialization, enables the debug, error, info, trace, warn macros
        log_rot::setup_logger().unwrap();
        //engine inicialization

        let mut engine = RotEngine {
            //event_buffer: Vec::new(),
            layer_stack: LayerStack::new(),
            renderer: ROT_Renderer::build(),

            event_receiver: None,
        };

        engine
    }

    pub fn close() {}

    pub fn run(&mut self) {
        self.renderer.run()
    }

    pub fn receive_events(&self) {
        loop {
            match self.event_receiver.as_ref().unwrap().recv() {
                Ok(ev) => {
                    debug!("Received");
                }
                Err(err) => {
                    error!("event bugged = {}", err);
                    break;
                }
            }
        }
    }
}
