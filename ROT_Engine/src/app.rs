//! # ROT_GameEngine
//! this is where the magic happens

use log::{debug, error, info, trace, warn};
mod log_rot;

/// This struct will define all the engine behavior.
/// Everything happens inside it.
pub struct RotEngine {}

impl RotEngine {
    pub fn run() {
        log_rot::setup_logger().unwrap();

        trace!("Trace message");
        debug!("Debug message");
        info!("Info message");
        warn!("Warning message");
        error!("Error message");
    }
}
