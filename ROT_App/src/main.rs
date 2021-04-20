//this file is just an app to costumize the engine behavior
//in ways i'm not sure right now =)

use futures::executor::block_on;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use rot::ROT_Engine;
use rot_events::Event_Base::Event;
use rot_events::MouseInput::TypeOfMouseEvent;

use nalgebra as na;
use nalgebra::Vector3;
use rot_wgpu::{
    rot_primitives::{Model, Texture, Vertex},
    Renderer,
};

fn main() {
    let game = Game::build_boxed("Renderer Teste".to_string());
    let mut engine = block_on(ROT_Engine::build([1280, 720], game));
    engine.run();

    debug!("End of Run");
}

struct Game {
    name: String,

    //Models
    textures: Vec<Texture>,
    models: Vec<Model>,

    //input
    mouse_pos: (f64, f64),
    space_toggle: bool,
}

impl Game {
    fn build_boxed(name: String) -> Box<Self> {
        let game = Game {
            name,
            textures: Vec::new(),
            models: Vec::new(),
            mouse_pos: (0.0, 0.0),
            space_toggle: false,
        };

        Box::new(game)
    }
}

impl rot_layer::Layer for Game {
    fn on_attach(&mut self, renderer: &Renderer) {
        let texture_a = Texture::build(
            "test",
            std::path::PathBuf::from("texture/happy-tree.png"),
            renderer,
        );

        let texture_b = Texture::build(
            "test",
            std::path::PathBuf::from("texture/america.png"),
            renderer,
        );

        let vertices = vec![
            Vertex {
                position: [-0.0868241, 0.49240386, 0.0],
                tex_coords: [0.4131759, 0.00759614],
            }, // A
            Vertex {
                position: [-0.49513406, 0.06958647, 0.0],
                tex_coords: [0.0048659444, 0.43041354],
            }, // B
            Vertex {
                position: [-0.21918549, -0.44939706, 0.0],
                tex_coords: [0.28081453, 0.949397057],
            }, // C
            Vertex {
                position: [0.35966998, -0.3473291, 0.0],
                tex_coords: [0.85967, 0.84732911],
            }, // D
            Vertex {
                position: [0.44147372, 0.2347359, 0.0],
                tex_coords: [0.9414737, 0.2652641],
            }, // E
        ];

        let indices: Vec<u16> = vec![0, 1, 4, 1, 2, 4, 2, 3, 4];

        let model = Model::new(renderer, vertices, indices);

        self.textures.push(texture_a);
        self.textures.push(texture_b);
        self.models.push(model);
    }

    fn on_event(&mut self, event: &Event) {
        match event {
            Event::MouseInput(ev) => match ev.ev_type {
                TypeOfMouseEvent::Button => {}
                TypeOfMouseEvent::Wheel => {}
                TypeOfMouseEvent::Movement => {
                    info!("{:?}", ev);

                    self.mouse_pos = (
                        ev.mouse_movement.as_ref().unwrap().position.x / 1280 as f64,
                        ev.mouse_movement.as_ref().unwrap().position.y / 720 as f64,
                    )
                }
            },
            Event::KeyboardInput(ev) => match ev.virtual_keycode {
                None => {}
                Some(keycode) => match ev.state {
                    rot_events::Event_Base::State::Pressed => {}
                    rot_events::Event_Base::State::Released => match keycode {
                        rot_events::KeyboardInput::KeyCode::Space => {
                            self.space_toggle = !self.space_toggle
                        }
                        _ => {}
                    },
                },
            },
        }
    }

    fn on_update(&mut self, renderer: &mut Renderer, delta_time: f64) {
        let index = match self.space_toggle {
            true => 1 as usize,
            false => 0 as usize,
        };
        let clear_color = Vector3::new(self.mouse_pos.0, self.mouse_pos.1, 0.3);

        renderer.draw_frame(&self.textures[index], &self.models[0], clear_color);
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}
