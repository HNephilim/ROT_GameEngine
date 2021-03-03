//this file is just an app to costumize the engine behavior
//in ways i'm not sure right now =)
use log::{debug, error, info, trace, warn};
use rot::app;

fn main() {
    app::RotEngine::run();

    debug!("Ending");
    rot::print_something_nice();
}
