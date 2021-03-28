//this file is just an app to costumize the engine behavior
//in ways i'm not sure right now =)
use log::{debug, error, info, trace, warn};
use rot::RotEngine;

fn main() {
    let mut engine = RotEngine::build();
    engine.run();

    debug!("End of Run");
}
