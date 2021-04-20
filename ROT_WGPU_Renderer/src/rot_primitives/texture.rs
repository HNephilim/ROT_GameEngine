use crate::rot_primitives::vertex::Vertex;
use crate::Renderer;

pub struct Texture {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_module: wgpu::ShaderModule,
    pub fragment_module: wgpu::ShaderModule,
}

impl Texture {
    pub fn build(shader_name: &str, diffuse_src: std::path::PathBuf, renderer: &Renderer) -> Self {
        let diffuse_texture = Texture::upload_image(diffuse_src.clone(), renderer);
        let (viewer, sampler) = Texture::create_view_and_sampler(&diffuse_texture, renderer);

        let bind_group_layout = Texture::create_bind_group_layout(renderer);
        let bind_group =
            Texture::create_bind_group(renderer, &bind_group_layout, &viewer, &sampler);

        let (vertex_module, fragment_module) =
            Texture::create_shader_modules(renderer, shader_name);
        let render_pipeline_layout = Texture::create_pipeline_layout(renderer, &bind_group_layout);
        let render_pipeline = Texture::create_pipeline(
            renderer,
            &render_pipeline_layout,
            &vertex_module,
            &fragment_module,
        );

        Self {
            bind_group,
            bind_group_layout,
            render_pipeline_layout,
            render_pipeline,
            vertex_module,
            fragment_module,
        }
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
                    buffers: &[Vertex::desc()],
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

    fn create_pipeline_layout(
        renderer: &Renderer,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::PipelineLayout {
        // LAYOUT -----------------------------------------------------

        let pipeline = renderer
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        pipeline
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

    fn create_bind_group(
        renderer: &Renderer,
        layout: &wgpu::BindGroupLayout,
        viewer: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Diffuse bind group"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(viewer),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            })
    }

    fn create_bind_group_layout(renderer: &Renderer) -> wgpu::BindGroupLayout {
        renderer
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Diffuse Bind Group Layout"),
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
            })
    }

    fn create_view_and_sampler(
        texture: &wgpu::Texture,
        rederer: &Renderer,
    ) -> (wgpu::TextureView, wgpu::Sampler) {
        let diffuse_texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = rederer.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        (diffuse_texture_view, diffuse_sampler)
    }

    fn upload_image(path: std::path::PathBuf, renderer: &Renderer) -> wgpu::Texture {
        let diffuse_bytes = std::fs::read(path).unwrap();
        let diffuse_image = image::load_from_memory(diffuse_bytes.as_slice()).unwrap();
        let diffuse_rgba = diffuse_image.as_rgba8().unwrap();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };

        let diffuse_texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("diffuse texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        renderer.queue.write_texture(
            wgpu::TextureCopyView {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            diffuse_rgba,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * dimensions.0,
                rows_per_image: dimensions.1,
            },
            texture_size,
        );

        diffuse_texture
    }
}
