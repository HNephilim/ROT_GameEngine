//this file is just an app to costumize the engine behavior
//in ways i'm not sure right now =)
use log::{debug, error, info, trace, warn};
use rot::ROT_Engine;

fn main() {
    let mut engine = ROT_Engine::build([1280, 720]);
    engine.run();

    debug!("End of Run");
}
