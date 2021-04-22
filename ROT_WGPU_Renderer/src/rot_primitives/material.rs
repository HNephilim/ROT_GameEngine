use crate::rot_primitives::vertex::Vertex;
use crate::rot_primitives::Primitive;
use crate::Renderer;
use wgpu::{BindGroup, BindGroupLayout};

pub struct Material {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Material {
    pub fn build(diffuse_src: std::path::PathBuf, renderer: &Renderer) -> Self {
        let diffuse_texture = Material::upload_image(diffuse_src.clone(), renderer);
        let (viewer, sampler) = Material::create_view_and_sampler(&diffuse_texture, renderer);

        let bind_group_layout = Material::create_bind_group_layout(renderer);
        let bind_group =
            Material::create_bind_group(renderer, &bind_group_layout, &viewer, &sampler);

        Self {
            bind_group,
            bind_group_layout,
        }
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

impl Primitive for Material {
    fn get_bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
