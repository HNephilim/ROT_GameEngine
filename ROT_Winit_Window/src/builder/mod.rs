use crate::ROT_Window;
use log::{debug, error, info, trace, warn};
use rot_events::{
    ROT_Event_Base::{ROT_Event, ROT_State},
    ROT_MouseInput::{ROT_Button, ROT_MouseButton, ROT_MouseEvent},
};

use std::sync::Arc;
use winit::{event::WindowEvent, window::WindowBuilder};

use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone)]
pub struct ROT_WindowBuilder {
    title_bar: String,
}

impl ROT_WindowBuilder {
    pub fn new(title_bar: String) -> Self {
        ROT_WindowBuilder {
            title_bar: title_bar.clone(),
        }
    }

    pub fn get_title_bar(&self) -> String {
        self.title_bar.clone()
    }

    pub fn build(
        window_builder: Arc<ROT_WindowBuilder>,
    ) -> (
        Receiver<Box<dyn ROT_Event + Send>>,
        Arc<ROT_Window<'static>>,
    ) {
        let (ev_tx, ev_rx) = channel();
        let (window_tx, window_rx) = channel();

        let event_thread_builder = std::thread::Builder::new().name("Event Thread".to_string());
        info!("Spawning Event Thread");
        let event_thread_handle = event_thread_builder
            .spawn(move || {
                let mut window = ROT_Window::build(window_builder.as_ref());
                //window_tx.send(Arc::new(window));
                window.run(ev_tx);
            })
            .unwrap();

        info!("Event thread finished");
        let rot_window = window_rx.recv().unwrap();
        (ev_rx, rot_window)
    }
}
