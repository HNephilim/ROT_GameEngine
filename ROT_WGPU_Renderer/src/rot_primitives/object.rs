use crate::rot_primitives::mesh::Mesh;
use crate::rot_primitives::Instance;

pub struct Object {
    pipeline: Option<wgpu::RenderPipeline>,
    mesh: Option<Mesh>,
    material: Option<Mesh>,

    instances: Vec<Instance>,
}
