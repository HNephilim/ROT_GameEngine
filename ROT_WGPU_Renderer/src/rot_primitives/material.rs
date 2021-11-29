use crate::rot_primitives::Primitive;
use crate::Renderer;
use wgpu::{BindGroup, BindGroupLayout};

pub struct Material {
    pub name: String,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,

    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Material {
    pub fn build(diffuse_src: std::path::PathBuf, renderer: &Renderer, name: &str) -> Self {
        let diffuse_texture = Material::upload_image(diffuse_src.clone(), renderer, name);
        let (texture_view, sampler) =
            Material::create_view_and_sampler(&diffuse_texture, renderer, name);

        let bind_group_layout = Material::get_bind_group_layout(renderer);
        let bind_group = Material::create_bind_group(
            renderer,
            &bind_group_layout,
            &texture_view,
            &sampler,
            name,
        );

        let fragment_module = Material::load_shader(renderer, name);

        Self {
            name: name.to_string(),
            bind_group,
            bind_group_layout,
            texture_view,
            sampler,
            fragment_module,
        }
    }

    fn create_bind_group(
        renderer: &Renderer,
        layout: &wgpu::BindGroupLayout,
        viewer: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        name: &str,
    ) -> wgpu::BindGroup {
        renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{} Diffuse Bind Group", name)),
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

    pub(crate) fn get_bind_group_layout(renderer: &Renderer) -> wgpu::BindGroupLayout {
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
        renderer: &Renderer,
        name: &str,
    ) -> (wgpu::TextureView, wgpu::Sampler) {
        let diffuse_texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!(" {} Sampler", name)),
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

    fn load_shader(renderer: &Renderer, name: &str) -> wgpu::ShaderModule {
        // FRAGMENT ----------------------------------------------------
        let fragment_path = format!("shaders/test.frag.spv");
        let frag_bytes = std::fs::read(fragment_path.clone()).unwrap();
        let fragment_module = renderer
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some(fragment_path.as_str()),
                source: wgpu::util::make_spirv(&frag_bytes),
                flags: wgpu::ShaderFlags::VALIDATION,
            });

        fragment_module
    }

    fn upload_image(path: std::path::PathBuf, renderer: &Renderer, name: &str) -> wgpu::Texture {
        let diffuse_bytes = std::fs::read(path).unwrap();
        let diffuse_image = image::load_from_memory(diffuse_bytes.as_slice()).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };

        let diffuse_texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{} diffuse texture", name)),
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
            diffuse_rgba.as_raw(),
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
