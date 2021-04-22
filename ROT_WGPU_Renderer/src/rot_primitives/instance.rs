use nalgebra as na;
use nalgebra::UnitQuaternion;

pub struct Instance{
    pub position: na::Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,

    pub uniform: InstanceUniform
}

impl Instance{
    pub fn update(&mut self){
        let transform = na::Matrix4::new_translation(&self.position) * na::Matrix4::from(self.rotation);

        self.uniform.transform = transform.into();


    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceUniform>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: 0,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                },
            ],
        }
    }
}


#[repr(C)]
#[derive(Default,Debug, Copy, Clone)]
pub struct InstanceUniform{
    pub transform: [[f32; 4]; 4],
}

unsafe impl bytemuck::Zeroable for InstanceUniform {}
unsafe impl bytemuck::Pod for InstanceUniform {}