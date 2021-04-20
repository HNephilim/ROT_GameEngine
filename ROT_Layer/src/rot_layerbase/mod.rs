#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use rot_events::Event_Base::Event;
use rot_wgpu::Renderer;

pub trait Layer {
    fn on_attach(&mut self, renderer: &Renderer);

    fn on_event(&mut self, event: &Event);

    fn on_update(&mut self, renderer: &mut Renderer, delta_time: f64);

    fn get_name(&self) -> &String;
}
