use crate::rot_primitives::{Camera, Instance, Material, Primitive, Vertex};
use crate::Renderer;

pub struct Pipeline {
    pub render_pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_module: wgpu::ShaderModule,
}

impl Pipeline {
    pub fn new(
        renderer: &Renderer,
        fragment_module: &wgpu::ShaderModule,
        shader_name: &str,
    ) -> Self {
        let render_pipeline_layout = Pipeline::create_pipeline_layout(renderer);

        let vertex_module = Pipeline::load_fragment_shader_module(renderer, shader_name);

        let render_pipeline = Pipeline::create_pipeline(
            renderer,
            &render_pipeline_layout,
            &vertex_module,
            &fragment_module,
        );

        Self {
            render_pipeline_layout,
            render_pipeline,
            vertex_module,
        }
    }

    fn load_fragment_shader_module(renderer: &Renderer, name: &str) -> wgpu::ShaderModule {
        // VERTEX ----------------------------------------------------
        let vertex_path = format!("shaders/test.vert.spv");
        let vert_bytes = std::fs::read(vertex_path.clone()).unwrap();
        let vertex_module = renderer
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some(vertex_path.as_str()),
                source: wgpu::util::make_spirv(&vert_bytes),
                flags: wgpu::ShaderFlags::VALIDATION,
            });

        vertex_module
    }

    fn create_pipeline_layout(renderer: &Renderer) -> wgpu::PipelineLayout {
        let material_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("Test Diffuse Bind Group Layout")),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                filtering: true,
                                comparison: false,
                            },
                            count: None,
                        },
                    ],
                });

        let camera_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera Uniform Bind Group"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        // LAYOUT -----------------------------------------------------

        let pipeline = renderer
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&material_layout, &camera_layout],
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
