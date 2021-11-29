#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use wgpu::util::DeviceExt;

use winit::window::Window;

pub mod rot_primitives;
use crate::rot_primitives::Object;
use rot_primitives::{Camera, DepthBufferTexture, Material, Mesh, Vertex};

pub mod rot_pipeline;

use nalgebra as na;

pub struct Renderer<'a> {
    //DepthBuffer
    depth_buffer: DepthBufferTexture,

    //Command Buffer
    command_buffer: Option<Vec<wgpu::CommandBuffer>>,
    frame: Option<wgpu::SwapChainTexture>,

    //Present Stuff
    pub render_pass: Option<wgpu::RenderPass<'a>>,
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swapchain_descriptor: wgpu::SwapChainDescriptor,
    swapchain: wgpu::SwapChain,

    //Window & EventLoop
    pub size: winit::dpi::PhysicalSize<u32>,

    clear_color: na::Vector3<f64>,
    camera: Option<&'a Camera>,
}

impl<'a> Renderer<'a> {
    pub async fn build(window: &Window) -> Self {
        info!("Building WGPU Renderer");

        trace!("Building Window and Event Loop");
        let size = window.inner_size();

        trace!("Building Instance");
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        trace!("Building Surface");
        let surface = unsafe { instance.create_surface(window) };

        trace!("Getting Physical Device");
        let adapter = Renderer::get_adapter(&instance, &surface).await;

        trace!("Creating Device and Queue");
        let (device, queue) = Renderer::create_device_queue(&adapter).await;

        trace!("Creating Swapchain");
        let (swapchain_descriptor, swapchain) =
            Renderer::create_swapchain(&device, &adapter, &surface, &size);

        trace!("Creating DepthBuffer");
        let depth_buffer = DepthBufferTexture::new(&device, &swapchain_descriptor, "depth_buffer");

        info!("Renderer Built");

        let clear_color = na::Vector3::new(0.0, 0.0, 0.0);

        Renderer {
            depth_buffer,
            command_buffer: Some(Vec::new()),
            frame: None,
            render_pass: None,
            surface,
            device,
            queue,
            swapchain_descriptor,
            swapchain,
            size,
            clear_color,
            camera: None,
        }
    }

    pub fn destroy(&mut self) {}

    pub fn set_clear_color(&mut self, color_rgb: [f64; 3]) {
        self.clear_color = na::Vector3::new(color_rgb[0], color_rgb[1], color_rgb[2]);
    }

    pub fn set_camera(&mut self, camera: &Camera) {
        self.camera = Some(camera);
    }
}

impl Renderer {
    pub(crate) fn get_render_pass(&mut self) -> &mut wgpu::RenderPass {
        self.render_pass.as_mut().unwrap()
    }

    pub(crate) fn draw_frame(
        &mut self,
        object: &Object,
        render_pipeline: &rot_pipeline::Pipeline,
    ) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swapchain.get_current_frame()?.output;

        let mut cmd_encoder =
            vec![self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                })];

        let mut render_pass = cmd_encoder[0].begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: self.clear_color[0],
                        g: self.clear_color[1],
                        b: self.clear_color[2],
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_buffer.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        self.render_pass = Some(render_pass);

        let mut cmd_encoder_finished = cmd_encoder
            .into_iter()
            .map(|encoder| encoder.finish())
            .collect::<Vec<_>>();

        self.command_buffer
            .as_mut()
            .unwrap()
            .append(&mut cmd_encoder_finished);
        self.frame = Some(frame);
        Ok(())
    }

    pub fn render(&mut self) {
        let frame = self.frame.take().unwrap();
        self.queue.submit(self.command_buffer.take().unwrap());

        self.command_buffer = Some(Vec::new());
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swapchain_descriptor.width = new_size.width;
        self.swapchain_descriptor.height = new_size.height;
        self.swapchain = self
            .device
            .create_swap_chain(&self.surface, &self.swapchain_descriptor);
        self.depth_buffer =
            DepthBufferTexture::new(&self.device, &self.swapchain_descriptor, "depth_buffer");
    }

    fn create_swapchain(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        surface: &wgpu::Surface,
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> (wgpu::SwapChainDescriptor, wgpu::SwapChain) {
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swapchain = device.create_swap_chain(surface, &sc_desc);

        (sc_desc, swapchain)
    }

    async fn create_device_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        (device, queue)
    }

    async fn get_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        let power_pref = wgpu::PowerPreference::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                compatible_surface: Some(surface),
            })
            .await
            .unwrap();

        adapter
    }
}
