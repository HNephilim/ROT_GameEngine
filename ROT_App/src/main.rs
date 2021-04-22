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

    debug!("End of Run");
}

struct Game {
    name: String,

    //Pipelines
    render_pipelines: Vec<rot::Pipeline>,

    //Models
    textures: Vec<rot::Texture>,
    models: Vec<rot::Model>,

    //camera
    cameras: Vec<rot::Camera>,


    //input
    mouse_pos: (f64, f64),
    space_toggle: bool,
}

impl Game {
    fn build_boxed(name: String) -> Box<Self> {
        
        let game = Game {
            name,
            render_pipelines: Vec::new(),
            textures: Vec::new(),
            models: Vec::new(),
            cameras: Vec::new(),
            mouse_pos: (0.0, 0.0),
            space_toggle: false,
        };

        Box::new(game)
    }
}

impl rot::Layer for Game {
    fn on_attach(&mut self, renderer: &mut rot::Renderer) {
        let texture_a = rot::Texture::build(
            std::path::PathBuf::from("texture/happy-tree.png"),
            renderer,
        );

        let texture_b = rot::Texture::build(
            std::path::PathBuf::from("texture/america.png"),
            renderer,
        );

        let vertices = vec![
            rot::Vertex {
                position: [-0.0868241, 0.49240386, 0.0],
                tex_coords: [0.4131759, 0.00759614],
            }, // A
            rot::Vertex {
                position: [-0.49513406, 0.06958647, 0.0],
                tex_coords: [0.0048659444, 0.43041354],
            }, // B
            rot::Vertex {
                position: [-0.21918549, -0.44939706, 0.0],
                tex_coords: [0.28081453, 0.949397057],
            }, // C
            rot::Vertex {
                position: [0.35966998, -0.3473291, 0.0],
                tex_coords: [0.85967, 0.84732911],
            }, // D
            rot::Vertex {
                position: [0.44147372, 0.2347359, 0.0],
                tex_coords: [0.9414737, 0.2652641],
            }, // E
        ];

        let indices: Vec<u16> = vec![0, 1, 4, 1, 2, 4, 2, 3, 4];

        let model = rot::Model::new(renderer, vertices, indices);

        let camera = rot::Camera::new(renderer,0.2,
                                       na::Point3::new(0.0, 1.0, 2.0),
                                       na::Point3::new(0.0, 0.0, 0.0, ),
                                       na::Vector3::y(),
                                       (1280/720) as f32,
                                       45.0,
                                       0.1,
                                       100.0);

        self.textures.push(texture_a);
        self.textures.push(texture_b);
        self.cameras.push(camera);
        self.models.push(model);


        self.render_pipelines = self.textures.iter().map(|tex| rot::Pipeline::new(renderer, tex, &self.cameras[0], "test")).collect::<Vec<_>>();
    }

    fn on_event(&mut self, event: &rot::Event) {
        match event {
            rot::Event::MouseMovement(ev) =>  {

                    self.mouse_pos = (
                        ev.position.x / 1280 as f64,
                        ev.position.y / 720 as f64,
                    )
            }
            rot::Event::KeyboardInput(ev) =>  {
                self.cameras[0].on_event(event);
                match ev.state{
                    rot::State::Pressed => {
                        match ev.virtual_keycode{
                            None => {}
                            Some(keycode) => {
                                match keycode{
                                    rot::KeyCode::Space => {
                                        self.space_toggle = !self.space_toggle
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    rot::State::Released => {}
                }
            }
            _ => {}
        }
    }

    fn on_update(&mut self, renderer: &mut rot::Renderer, delta_time: f64) {
        self.cameras[0].on_update(renderer);

        let index = match self.space_toggle {
            true => 1 as usize,
            false => 0 as usize,
        };
        let clear_color = na::Vector3::new(self.mouse_pos.0, self.mouse_pos.1, 0.3);

        renderer.draw_frame(&self.textures[index], &self.models[0], &self.cameras[0], &self.render_pipelines[index], clear_color);
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}
