use crate::Renderer;
use crate::rot_primitives::{Vertex, Texture, Primitive, Camera, Instance};

pub struct Pipeline{
    pub render_pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_module: wgpu::ShaderModule,
    pub fragment_module: wgpu::ShaderModule,
}



impl Pipeline{
    pub fn new(renderer: &Renderer, texture: &Texture, camera: &Camera, shader_name: &str) -> Self {

        let render_pipeline_layout = Pipeline::create_pipeline_layout(renderer, texture, camera);

        let (vertex_module, fragment_module) =
            Pipeline::create_shader_modules(renderer, shader_name);

        let render_pipeline = Pipeline::create_pipeline(renderer, &render_pipeline_layout, &vertex_module, &fragment_module);

        Self { render_pipeline_layout, render_pipeline, vertex_module, fragment_module }
    }

    fn create_shader_modules(
        renderer: &Renderer,
        name: &str,
    ) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
        // VERTEX ----------------------------------------------------
        let vertex_path = format!("shaders/{}.vert.spv", name);
        let vert_bytes = std::fs::read(vertex_path.clone()).unwrap();
        let vertex_module = renderer
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some(vertex_path.as_str()),
                source: wgpu::util::make_spirv(&vert_bytes),
                flags: wgpu::ShaderFlags::VALIDATION,
            });

        // FRAGMENT --------------------------------------------------
        let frag_path = format!("shaders/{}.frag.spv", name);
        let frag_bytes = std::fs::read(frag_path.clone()).unwrap();
        let frag_module = renderer
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some(vertex_path.as_str()),
                source: wgpu::util::make_spirv(&frag_bytes),
                flags: wgpu::ShaderFlags::VALIDATION,
            });

        (vertex_module, frag_module)
    }

    fn create_pipeline_layout(
        renderer: &Renderer,
        texture: &Texture,
        camera: &Camera
    ) -> wgpu::PipelineLayout {
        // LAYOUT -----------------------------------------------------

        let pipeline = renderer
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    texture.get_bind_group_layout(),
                    camera.get_bind_group_layout()
                ],
                push_constant_ranges: &[],
            });

        pipeline
    }

    fn create_pipeline(
        renderer: &Renderer,
        render_pipeline_layout: &wgpu::PipelineLayout,
        vertex_module: &wgpu::ShaderModule,
        fragment_module: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        // CREATE -------------------------------------------------------
        renderer
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: vertex_module,
                    entry_point: "main",
                    buffers: &[Vertex::desc(), Instance::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: fragment_module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: renderer.swapchain_descriptor.format,
                        alpha_blend: wgpu::BlendState::REPLACE,
                        color_blend: wgpu::BlendState::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::Back,
                    polygon_mode: wgpu::PolygonMode::Fill,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            })
    }
}