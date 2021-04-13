//this file is just an app to costumize the engine behavior
//in ways i'm not sure right now =)

#[allow(unused_imports)]
#[allow(non_camel_case_types)]
use log::{debug, error, info, trace, warn};
use rot::ROT_Engine;

fn main() {
    let mut engine = ROT_Engine::build([1280, 720]);
    engine.run();
    engine.close();

    debug!("End of Run");
}
