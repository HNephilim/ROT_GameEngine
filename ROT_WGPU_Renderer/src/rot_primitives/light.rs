use crate::rot_pipeline::{Pipeline, PipelineBuilder};
use crate::rot_primitives::{Instance, Object, Vertex};
use crate::Renderer;
use nalgebra as na;
use wgpu::util::DeviceExt;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct LightUniform {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
}

unsafe impl bytemuck::Pod for LightUniform {}
unsafe impl bytemuck::Zeroable for LightUniform {}

impl LightUniform {
    pub fn build(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            _padding: 0,
            color,
        }
    }
}

pub struct Light {
    name: String,

    uniform: LightUniform,

    model: Object,

    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,

    pub pipeline: Pipeline,
}

impl Light {
    pub fn new(renderer: &Renderer, position: [f32; 3], color: [f32; 3], name: &str) -> Self {
        let uniform = LightUniform::build(position, color);

        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} light buffer", name)),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        let bind_group_layout = Light::get_bind_group_layout(renderer);

        let bind_group = Light::create_bind_group(renderer, &bind_group_layout, &buffer, name);

        let pipeline_buider = PipelineBuilder {
            name: format!("{} pipeline", name).as_str(),
            uniform_material: false,
            uniform_camera: true,
            uniform_light: true,
            vertex_shader_path: "shaders/light.vert.spv",
            fragment_shader_path: "shaders/light.frag.spv",
            vertex_buffer_layout: vec![Vertex::desc()],
        };

        let pipeline = Pipeline::new(renderer, pipeline_buider);

        let object = Object::load(renderer, "model/light/Bulbs.obj", "light");

        Self {
            name: name.to_string(),
            uniform,
            model: object,
            buffer,
            bind_group,
            bind_group_layout,
            pipeline,
        }
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        renderer.draw_frame(&self.model, &self.pipeline);
    }

    pub fn on_update(&mut self, renderer: &Renderer) {
        let old_position: na::Vector3<f32> = self.uniform.position.into();

        self.uniform.position = (na::UnitQuaternion::from_axis_angle(
            na::Vector3::y_axis(),
            std::f32::consts::FRAC_PI_8,
        ) * old_position)
            .into();

        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub(crate) fn get_bind_group_layout(renderer: &Renderer) -> wgpu::BindGroupLayout {
        renderer
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(" Camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
    }

    fn create_bind_group(
        renderer: &Renderer,
        bind_group_layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer,
        name: &str,
    ) -> wgpu::BindGroup {
        renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{} Uniform Bind Group", name)),
                layout: bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            })
    }
}
