use nalgebra as na;
use nalgebra::UnitQuaternion;

pub struct Instance {
    pub isometry: na::Isometry3<f32>,

    pub uniform: InstanceUniform,
}

impl Instance {
    pub fn new(isometry: na::Isometry3<f32>) -> Self {
        Self {
            isometry,
            uniform: InstanceUniform {
                model: isometry.to_homogeneous().into(),
            },
        }
    }

    pub fn update(&mut self) {
        self.uniform.model = self.isometry.to_homogeneous().into();
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceUniform>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
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

impl Default for Instance {
    fn default() -> Self {
        let default_isometry =
            na::Isometry3::new(na::Vector3::<f32>::new(0.0, 0.0, 0.0), na::Vector3::y());

        Instance::new(default_isometry)
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct InstanceUniform {
    pub model: [[f32; 4]; 4],
}

unsafe impl bytemuck::Zeroable for InstanceUniform {}
unsafe impl bytemuck::Pod for InstanceUniform {}
