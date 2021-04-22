mod camera;
mod light;
mod model;
mod texture;
mod vertex;
mod instance;

pub use model::Model;
pub use texture::Texture;
pub use vertex::Vertex;
pub use camera::Camera;
pub use instance::Instance;


pub trait Primitive{
    fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout;

    fn get_bind_group(&self) -> &wgpu::BindGroup;
}