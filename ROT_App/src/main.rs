//this file is just an app to costumize the engine behavior
//in ways i'm not sure right now =)

use futures::executor::block_on;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use rot::prelude as rot;

use nalgebra as na;

fn main() {
    let game = Game::build_boxed("Renderer Teste".to_string());
    let mut engine = block_on(rot::ROT_Engine::build([1280, 720], game));
    engine.run();
}

struct Game {
    name: String,

    //Models
    object: Vec<rot::Object>,

    //camera
    cameras: Vec<rot::Camera>,

    //light
    lights: Vec<rot::Light>,

    //input
    mouse_pos: (f64, f64),
    space_toggle: bool,
}

impl Game {
    fn build_boxed(name: String) -> Box<Self> {
        let game = Game {
            name,

            object: Vec::new(),
            cameras: Vec::new(),
            lights: Vec::new(),
            mouse_pos: (0.0, 0.0),
            space_toggle: false,
        };

        Box::new(game)
    }
}

impl rot::Layer for Game {
    fn on_attach(&mut self, renderer: &mut rot::Renderer) {
        let num_of_instances_per_row: u32 = 10;
        let instance_displacement: f32 = 3.0;

        let isometry_vec = (0..num_of_instances_per_row)
            .flat_map(|z| {
                (0..num_of_instances_per_row).map(move |x| {
                    let x =
                        instance_displacement * (x as f32 - num_of_instances_per_row as f32 / 2.0);
                    let z =
                        instance_displacement * (z as f32 - num_of_instances_per_row as f32 / 2.0);

                    let translation: na::Vector3<f32> = na::Vector3::new(x as f32, 0.0, z as f32);

                    let axisangle = if translation == na::Vector3::new(0.0, 0.0, 0.0) {
                        na::Vector3::z() * 0.0
                    } else {
                        translation.normalize() * std::f32::consts::PI / 4.0
                    };

                    if translation == na::Vector3::<f32>::new(0.0, 0.0, 0.0) {
                        log::info!("{}", axisangle);
                    }

                    na::Isometry3::new(translation, axisangle)
                })
            })
            .collect::<Vec<_>>();

        let mut object = rot::Object::load(renderer, "model/cube/cube.obj", "cube");

        object.set_instance(renderer, isometry_vec);

        self.object.push(object);

        let camera = rot::Camera::new(
            renderer,
            0.5,
            na::Point3::new(0.0, 1.0, 2.0),
            na::Point3::new(0.0, 0.0, 0.0),
            na::Vector3::y(),
            (renderer.swapchain_descriptor.width as f32
                / renderer.swapchain_descriptor.height as f32),
            std::f32::consts::FRAC_PI_4,
            0.01,
            100.0,
        );
        self.cameras.push(camera);

        let light = rot::Light::new(renderer, [2.0, 2.0, 2.0], [1.0, 1.0, 1.0], "light");

        self.lights.push(light);
    }

    fn on_event(&mut self, event: &rot::Event) {
        match event {
            rot::Event::MouseMovement(ev) => {
                self.mouse_pos = (ev.position.x / 1280 as f64, ev.position.y / 720 as f64)
            }
            rot::Event::KeyboardInput(ev) => {
                self.cameras[0].on_event(event);
                match ev.state {
                    rot::State::Pressed => match ev.virtual_keycode {
                        None => {}
                        Some(keycode) => match keycode {
                            rot::KeyCode::Space => self.space_toggle = !self.space_toggle,
                            _ => {}
                        },
                    },
                    rot::State::Released => {}
                }
            }
            _ => {}
        }
    }

    fn on_update(&mut self, renderer: &mut rot::Renderer, delta_time: f64) {
        let clear_color = (self.mouse_pos.0, self.mouse_pos.1, 0.2);
        renderer.set_clear_color([clear_color.0, clear_color.1, clear_color.2])

        for object in self.object.iter_mut() {
            object.on_update(renderer)
        }
        for camera in self.cameras.iter_mut() {
            camera.on_update(renderer);
        }
        renderer.set_camera(&self.cameras[0]);

        for object in self.object.iter() {
            object.draw(renderer)
        }
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}
