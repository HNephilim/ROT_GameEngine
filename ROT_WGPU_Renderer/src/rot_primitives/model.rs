use crate::rot_primitives::vertex::Vertex;
use crate::Renderer;
use nalgebra as na;

use wgpu::util::DeviceExt;
use crate::rot_primitives::instance::{Instance, InstanceUniform};



pub struct Model {
    pub index_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub instance_buffer:  wgpu::Buffer,
    size: usize,

    pub instances: Vec<Instance>
}

impl Model {
    pub fn new(renderer: &Renderer, vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        let mut instances = Model::build_instances();
        let instance_data = Model::get_instances_data(&instances);

        let instance = renderer
        .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsage::VERTEX,
            });

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
            instance_buffer: instance,
            size: indices.len(),
            instances
        }
    }

    fn get_instances_data(instances: &Vec<Instance>) -> Vec<InstanceUniform>{
        instances.iter().map(|instance|{
            instance.uniform
        }).collect()
    }

    fn build_instances() -> Vec<Instance>{
        let num_of_instances_per_row: u32 = 10;
        let instance_displacement: na::Vector3<f32> = na::Vector3::new(num_of_instances_per_row as f32 * 0.5, 0.0, num_of_instances_per_row as f32 * 0.5);


         (0..num_of_instances_per_row).flat_map(|z| {
            (0..num_of_instances_per_row).map(move |x|{
                let position: na::Vector3<f32> = na::Vector3::new(x as f32, 0.0, z as f32) - instance_displacement;

                let rotation = if position == na::Vector3::new(0.0, 0.0, 0.0){
                    na::UnitQuaternion::new(na::Vector3::z() * 0.0)
                } else{
                    na::UnitQuaternion::new(position.normalize() * (45.0/180.0)* std::f32::consts::PI)
                };

               let mut instance = Instance{
                    position,
                    rotation,
                    uniform: Default::default()
                };

                instance.update();
                instance

            })
        }).collect::<Vec<_>>()


    }



    pub fn len(&self) -> u32 {
        self.size as u32
    }
}
