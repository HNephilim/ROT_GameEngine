use crate::rot_primitives::{Camera, Instance, Light, Material, Primitive, Vertex};
use crate::Renderer;

pub struct Pipeline {
    pub render_pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_module: wgpu::ShaderModule,
    pub fragment_module: wgpu::ShaderModule,
}

impl Pipeline {
    pub fn new(renderer: &Renderer, pipeline_builder: PipelineBuilder) -> Self {
        let render_pipeline_layout = Pipeline::create_pipeline_layout(renderer, &pipeline_builder);

        let (vertex_module, fragment_module) =
            Pipeline::load_vertex_shader_module(renderer, &pipeline_builder);

        let render_pipeline = Pipeline::create_pipeline(
            renderer,
            &render_pipeline_layout,
            &vertex_module,
            &fragment_module,
            &pipeline_builder,
        );

        Self {
            render_pipeline_layout,
            render_pipeline,
            vertex_module,
            fragment_module,
        }
    }

    fn load_vertex_shader_module(
        renderer: &Renderer,
        pipeline_builder: &PipelineBuilder,
    ) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
        // VERTEX ----------------------------------------------------
        let vertex_path = pipeline_builder.vertex_shader_path;
        let vert_bytes = std::fs::read(vertex_path.clone()).unwrap();
        let vertex_module = renderer
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some(vertex_path.as_str()),
                source: wgpu::util::make_spirv(&vert_bytes),
                flags: wgpu::ShaderFlags::VALIDATION,
            });

        let frag_path = pipeline_builder.fragment_shader_path;
        let frag_bytes = std::fs::read(frag_path.clone()).unwrap();
        let frag_module = renderer
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some(frag_path),
                source: wgpu::util::make_spirv(&frag_bytes),
                flags: wgpu::ShaderFlags::VALIDATION,
            });

        (vertex_module, frag_module)
    }

    fn create_pipeline_layout(
        renderer: &Renderer,
        pipeline_builder: &PipelineBuilder,
    ) -> wgpu::PipelineLayout {
        let mut bind_gruop_layouts = Vec::new();

        if pipeline_builder.uniform_material {
            let material_layout = Material::get_bind_group_layout(renderer);
            bind_gruop_layouts.push(material_layout);
        }

        if pipeline_builder.uniform_camera {
            let camera_layout = Camera::get_bind_group_layout(renderer);
            bind_gruop_layouts.push(camera_layout);
        }

        if pipeline_builder.uniform_light {
            let light_layout = Light::get_bind_group_layout(renderer);
            bind_gruop_layouts.push(light_layout);
        }

        let pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(format!("{} {}", pipeline_builder.name, "layout").as_str()),
                    bind_group_layouts: &[bind_gruop_layouts.as_ref()],
                    push_constant_ranges: &[],
                });

        pipeline_layout
    }

    fn create_pipeline(
        renderer: &Renderer,
        render_pipeline_layout: &wgpu::PipelineLayout,
        vertex_module: &wgpu::ShaderModule,
        fragment_module: &wgpu::ShaderModule,
        pipeline_builder: &PipelineBuilder,
    ) -> wgpu::RenderPipeline {
        // CREATE -------------------------------------------------------
        renderer
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(pipeline_builder.name),
                layout: Some(render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: vertex_module,
                    entry_point: "main",
                    buffers: &pipeline_builder.vertex_buffer_layout.as_slice(),
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                    clamp_depth: false,
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            })
    }
}

pub struct PipelineBuilder<'a> {
    pub name: &'a str,

    pub uniform_material: bool,
    pub uniform_camera: bool,
    pub uniform_light: bool,

    pub vertex_shader_path: &'a str,
    pub fragment_shader_path: &'a str,

    pub vertex_buffer_layout: Vec<wgpu::VertexBufferLayout<'a>>,
}
