use ash::vk;
use nalgebra as na;
use std::mem::size_of;

#[repr(C)]
pub struct Vertex {
    position: na::Vector2<f32>,
    color: na::Vector3<f32>,
}

impl<'a> Vertex {
    pub fn get_binding_description() -> Vec<vk::VertexInputBindingDescription> {
        let binding_description = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build();

        let binding_description_vec = vec![binding_description];

        binding_description_vec
    }

    pub fn get_attribute_description() -> Vec<vk::VertexInputAttributeDescription> {
        let position_attribute_desc = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0)
            .build();

        let color_attribute_desc = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(8)
            .build();

        let attribute_desc_vec = vec![position_attribute_desc, color_attribute_desc];

        attribute_desc_vec
    }

    pub fn get_exemple_vector() -> Vec<Vertex> {
        let a = Vertex {
            position: na::Vector2::new(0.0, -0.5),
            color: na::Vector3::new(1.0, 0.0, 0.0),
        };
        let b = Vertex {
            position: na::Vector2::new(0.5, 0.5),
            color: na::Vector3::new(0.0, 1.0, 0.0),
        };
        let c = Vertex {
            position: na::Vector2::new(-0.0, 0.5),
            color: na::Vector3::new(0.0, 0.0, 1.0),
        };

        let vec = vec![a, b, c];

        vec
    }
}
