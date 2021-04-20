pub mod ev_translator_winit;
mod rot_events;

pub use ev_translator_winit as EventTranslator;
pub use rot_events::Event_Base;
pub use rot_events::KeyboardInput;
pub use rot_events::MouseInput;
