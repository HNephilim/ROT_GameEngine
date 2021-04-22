use nalgebra as na;
use crate::Renderer;
use wgpu::util::DeviceExt;
use bytemuck::{Zeroable, Pod};
use crate::rot_primitives::Primitive;
use wgpu::{BindGroup, BindGroupLayout};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: na::Matrix4<f32> = na::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);



pub struct Camera {
    camera_status: CameraStatus,

    controller: CameraController,

    pub uniform: CameraUniform,
    

    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Camera {
    pub fn new(renderer: &Renderer,speed: f32, eye: na::Point3<f32>, target: na::Point3<f32>, up: na::Vector3<f32>, aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self{
        let camera_status = CameraStatus{
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar
        };
        let uniform = CameraUniform::new(&camera_status);

        let controller = CameraController::new(speed);

        let buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST
        });

        let bind_group_layout = Camera::create_bind_group_layout(renderer);

        let bind_group = Camera::create_bind_group(renderer, &bind_group_layout, &buffer);



        Self{
            camera_status,
            controller,
            uniform,
            buffer,
            bind_group,
            bind_group_layout,

        }
    }

    pub fn on_event(&mut self, event: &rot_events::event::Event){
        self.controller.on_event(event);
    }

    pub fn on_update(&mut self, renderer:&Renderer){
        self.controller.on_update(&mut self.camera_status);
        self.uniform.update(&self.camera_status);
        renderer.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    fn create_bind_group(renderer: &Renderer, bind_group_layout: &wgpu::BindGroupLayout, buffer: &wgpu::Buffer) -> wgpu::BindGroup{
        renderer.device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("Camera Uniform Bind Group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry{
                binding: 0,
                resource: buffer.as_entire_binding()
            }]
        })
    }

    fn create_bind_group_layout(renderer: &Renderer) -> wgpu::BindGroupLayout{
        renderer.device.create_bind_group_layout(&wgpu ::BindGroupLayoutDescriptor{
            label: Some("Camera Uniform Bind Group"),
            entries: &[wgpu::BindGroupLayoutEntry{
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }]
        })
    }
}

impl Primitive for Camera{
    fn get_bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}


pub struct CameraStatus{
    pub eye: na::Point3<f32>,
    pub target: na::Point3<f32>,
    pub up: na::Vector3<f32>,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[repr(C)]
#[derive(Default,Debug, Copy, Clone)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4 ]; 4]
}

unsafe impl Zeroable for CameraUniform {}
unsafe impl Pod for CameraUniform {}


impl CameraUniform{
    pub fn new(status: &CameraStatus) -> Self{
        let view = na::Matrix4::look_at_rh(&status.eye, &status.target, &status.up);
        let proj = na::Matrix4::new_perspective(status.aspect, status.fovy, status.znear, status.zfar);

        let matrix = OPENGL_TO_WGPU_MATRIX * proj * view;

        Self{
            view_proj: matrix.into()
        }
    }

    pub fn update(&mut self, camera:&CameraStatus){
        let view = na::Matrix4::look_at_rh(&camera.eye, &camera.target, &camera.up);
        let proj = na::Matrix4::new_perspective(camera.aspect, camera.fovy, camera.znear, camera.zfar);

        let matrix = OPENGL_TO_WGPU_MATRIX * proj * view;


        self.view_proj = matrix.into()

    }
}

pub struct CameraController{    
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }


    fn on_event(&mut self, event: &rot_events::event::Event){
        use rot_events::event::{Event, State};
        use rot_events::KeyboardInput::KeyCode;

        match event{
            Event::MouseButton(_) => {}
            Event::MouseWheel(_) => {}
            Event::MouseMovement(_) => {}
            Event::KeyboardInput(ev) => {
                match ev.state{
                    State::Pressed => {
                        match ev.virtual_keycode{
                            None => {}
                            Some(keycode) => {
                                match keycode {
                                    KeyCode::Space => {self.is_up_pressed = true}
                                    KeyCode::LAlt => {self.is_down_pressed = true}
                                    KeyCode::W => {self.is_forward_pressed = true}
                                    KeyCode::S => {self.is_backward_pressed = true}
                                    KeyCode::A => {self.is_left_pressed = true}
                                    KeyCode::D => {self.is_right_pressed = true}
                                    _ => {}
                                }
                            }
                        }
                    }
                    State::Released => {
                        match ev.virtual_keycode{
                            None => {}
                            Some(keycode) => {
                                match keycode {
                                    KeyCode::Space => {self.is_up_pressed = false}
                                    KeyCode::LAlt => {self.is_down_pressed = false}
                                    KeyCode::W => {self.is_forward_pressed = false}
                                    KeyCode::S => {self.is_backward_pressed = false}
                                    KeyCode::A => {self.is_left_pressed = false}
                                    KeyCode::D => {self.is_right_pressed = false}
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn on_update(&self, camera: &mut CameraStatus){
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_forward_pressed && forward_mag > self.speed{
            camera.eye += forward.normalize() * self.speed
        }
        if self.is_backward_pressed {
            camera.eye -= forward.normalize() * self.speed;
        }

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            let right = forward.normalize().cross(&camera.up);
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            let right = forward.normalize().cross(&camera.up);
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }

    }
}