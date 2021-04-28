use crate::Renderer;

pub(crate) struct DepthBufferTexture {
    pub depth_format: wgpu::TextureFormat,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl DepthBufferTexture {
    pub(crate) fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        name: &str,
    ) -> Self {
        let depth_format = wgpu::TextureFormat::Depth32Float;

        let size = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth_format,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        });

        let view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(name),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        Self {
            depth_format,
            view,
            sampler,
        }
    }
}
