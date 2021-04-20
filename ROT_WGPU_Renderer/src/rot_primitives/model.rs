use crate::rot_primitives::vertex::Vertex;
use crate::Renderer;

use wgpu::util::DeviceExt;

pub struct Model {
    pub index_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    size: usize,
}

impl Model {
    pub fn new(renderer: &Renderer, vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        let vertex = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices.as_slice()),
                usage: wgpu::BufferUsage::VERTEX,
            });

        let index = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices.as_slice()),
                usage: wgpu::BufferUsage::INDEX,
            });

        Self {
            index_buffer: index,
            vertex_buffer: vertex,
            size: indices.len(),
        }
    }

    pub fn len(&self) -> u32 {
        self.size as u32
    }
}
