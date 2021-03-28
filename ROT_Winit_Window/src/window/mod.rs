use log::{debug, error, info, trace, warn};

use crate::builder::ROT_WindowBuilder;
use rot_events::{
    ROT_EventTranslator,
    ROT_Event_Base::{ROT_Event, ROT_State},
    ROT_MouseInput::{ROT_Button, ROT_MouseButton, ROT_MouseEvent},
};
use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::platform::windows::EventLoopExtWindows;
use winit::{
    error::OsError,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct ROT_Window<'a> {
    title_bar: String,

    window: Option<Arc<Window>>,

    //EventBuffer
    event_buffer: Vec<WindowEvent<'a>>,
}

impl<'a> ROT_Window<'a> {
    pub fn build(window_builder: &ROT_WindowBuilder) -> Self {
        info!("WINIT WINDOWING BUILDING!");
        ROT_Window {
            title_bar: window_builder.get_title_bar(),
            window: None,
            event_buffer: Vec::new(),
        }
    }

    pub fn run(&mut self, event_sender: Sender<Box<dyn ROT_Event + Send>>) {
        trace!("Setting up event loop");
        let mut event_loop = EventLoop::new_any_thread();
        let window_builder = WindowBuilder::new().with_title(&self.title_bar);

        trace!("Creating Window");
        let window = window_builder.build(&event_loop).unwrap();
        self.window = Some(Arc::new(window));
        trace!("Window Creation Succesfull");

        trace!("Starting pulling events");

        event_loop.run_return(|event: Event<()>, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event, window_id } => {
                    if window_id == self.window.as_ref().unwrap().id() {
                        match event {
                            WindowEvent::CloseRequested => {
                                debug!("Close asked for");
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::MouseInput { .. }
                            | WindowEvent::MouseWheel { .. }
                            | WindowEvent::CursorMoved { .. } => {
                                let rot_event = ROT_EventTranslator::mouse_event(event);
                                event_sender.send(Box::new(rot_event.unwrap()));
                            }
                            WindowEvent::KeyboardInput { .. } => {
                                let rot_event = ROT_EventTranslator::keyboard_input_event(event);
                                event_sender.send(Box::new(rot_event.unwrap()));
                            }
                            _ => {}
                        }
                    }
                }

                _ => (),
            }
        });
    }
}
