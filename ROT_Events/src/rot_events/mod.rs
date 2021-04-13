#[allow(unused_imports)]
#[allow(non_camel_case_types)]
pub mod commom;
pub mod keyboard;
pub mod mouse;

pub use commom as ROT_Event_Base;
pub use keyboard as ROT_KeyboardInput;
pub use mouse as ROT_MouseInput;
