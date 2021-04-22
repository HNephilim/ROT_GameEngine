//! # ROT_GameEngine
//! this is where the magic happens

pub mod prelude;

use rot_wgpu::Renderer;

mod log_rot;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rot_layer;
use rot_layer::{Layer, LayerStack};

use rot_events::{EventTranslator, event::Event as RotEvent};

use rot_gui::Gui;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

/// This struct will define all the engine behavior.
/// Everything happens inside it.

pub struct Engine {
    //Buffer and Stacks
    layer_stack: LayerStack,

    //Renderer
    renderer: Renderer,

    //Event
    event_receiver: Option<Receiver<Arc<RotEvent>>>,
    event_buffer: Vec<RotEvent>,
    //Window
}

// public impl block
impl Engine {
    pub async fn build(window: &winit::window::Window) -> Self {
        //log inicialization, enables the debug, error, info, trace, warn macros
        log_rot::setup_logger().unwrap();
        //engine inicialization

        let mut layer_stack = LayerStack::new();
        let renderer = Renderer::build(window).await;
        let gui = Box::new(Gui::build("GUI".to_string(), window));
        layer_stack.push_indexed(gui, 0);

        let event_buffer = Vec::new();

        Self {
            layer_stack,
            renderer,

            event_receiver: None,
            event_buffer,
        }
    }

    pub fn close(&mut self) {
        self.renderer.destroy();
    }

    pub fn run(&mut self) {}

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
            layer.on_update(&mut self.renderer, delta_time)
        }
    }
}

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[allow(non_camel_case_types)]
pub struct ROT_Engine {
    event_loop: Option<EventLoop<()>>,
    window: Option<Window>,
    engine: Option<Engine>,
}

impl ROT_Engine {
    pub async fn build(dimensions: [u32; 2], game: Box<dyn rot_layer::Layer>) -> Self {
        let (window, event_loop) = ROT_Engine::create_window(dimensions);
        let mut engine = Engine::build(&window).await;
        engine.layer_stack.push_indexed(game, 1);

        Self {
            event_loop,
            window: Some(window),
            engine: Some(engine),
        }
    }

    fn create_window(dimension: [u32; 2]) -> (Window, Option<EventLoop<()>>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::PhysicalSize {
                width: dimension[0],
                height: dimension[1],
            })
            .with_title("A ROT_Engine")
            .with_resizable(true)
            .build(&event_loop)
            .unwrap();

        (window, Some(event_loop))
    }

    pub fn run(mut self) {
        let mut engine = self.engine.take().unwrap();
        for layer in engine.layer_stack.stack() {
            layer.on_attach(&mut engine.renderer)
        }

        let event_loop = self.event_loop.take().unwrap();
        let window = self.window.take().unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { .. } => {
                    let rot_event = EventTranslator::keyboard_input_event(event);
                    match rot_event {
                        None => {}
                        Some(ev) => engine.event_buffer.push(RotEvent::KeyboardInput(ev)),
                    }
                }
                WindowEvent::CursorMoved { .. } => {
                    let rot_event = EventTranslator::mouse_movement(event);
                    match rot_event {
                        None => {}
                        Some(ev) => engine.event_buffer.push(RotEvent::MouseMovement(ev)),
                    }}
                WindowEvent::MouseInput { .. } => {
                    let rot_event = EventTranslator::mouse_button(event);
                    match rot_event {
                        None => {}
                        Some(ev) => engine.event_buffer.push(RotEvent::MouseButton(ev)),
                    }
                }
                WindowEvent::MouseWheel { .. } => {
                    let rot_event = EventTranslator::mouse_wheel(event);
                    match rot_event {
                        None => {}
                        Some(ev) => engine.event_buffer.push(RotEvent::MouseWheel(ev)),
                    }
                }
                WindowEvent::Resized(size) => {
                    engine.renderer.resize(*size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    engine.renderer.resize(**new_inner_size);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                engine.dispatch_events();
                engine.update(0.5);
                engine.renderer.render();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        });
    }
}
