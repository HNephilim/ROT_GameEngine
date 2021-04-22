mod camera;
mod instance;
mod light;
mod material;
mod mesh;
mod object;
mod vertex;

pub use camera::Camera;
pub use instance::Instance;
pub use material::Material;
pub use mesh::Mesh;
pub use object::Object;
pub use vertex::Vertex;

pub trait Primitive {
    fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout;

    fn get_bind_group(&self) -> &wgpu::BindGroup;
}
