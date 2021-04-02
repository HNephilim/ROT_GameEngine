use log::{debug, error, info, trace, warn};

use rot_events::{ROT_EventTranslator, ROT_Event_Base::ROT_Event};

use std::borrow::Borrow;
use std::iter::Cloned;
use std::sync::Arc;

mod graphics_structs;
pub use graphics_structs::Vertex2D;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState, SubpassContents};
use vulkano::device::{Device, DeviceExtensions, Features, Queue, QueuesIter};
use vulkano::framebuffer::{Framebuffer, RenderPass, RenderPassDesc};
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice, QueueFamily};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform,
    Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_shaders;

use vulkano_win::{CreationError, VkSurfaceBuild};

use std::sync::mpsc::Sender;

use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};

pub struct ROT_Renderer {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface<Window>>,
    pub swapchain: Arc<Swapchain<Window>>,
    pub images: Vec<Arc<SwapchainImage<Window>>>,

    event_loop: Option<EventLoop<()>>,
}

impl ROT_Renderer {
    pub fn build(dimensions: [u32; 2]) -> Self {
        info!("VULKAN RENDERER BUILDING!");
        let rqd_instance_ext = vulkano_win::required_extensions();
        let instance = ROT_Renderer::create_instance(&rqd_instance_ext);

        let physical_device = ROT_Renderer::get_physical_device(&instance);

        let surface_and_eventloop = ROT_Renderer::create_surface(&instance, dimensions);
        let (surface, eventloop) = surface_and_eventloop;

        let queue_family = ROT_Renderer::get_queue_family(&physical_device, &surface);

        let device_and_queue =
            ROT_Renderer::get_device_and_queueiter(&physical_device, &queue_family);
        let (device, mut queue_iter) = device_and_queue;
        let queue = queue_iter.next().unwrap();

        trace!(
            "Getting queue id {} in family id {}",
            queue.id_within_family(),
            queue.family().id()
        );

        let swapchain_and_images =
            ROT_Renderer::create_swapchain(&physical_device, &surface, &device, &queue);
        let (swapchain, images) = swapchain_and_images;

        ROT_Renderer {
            instance,
            device,
            queue,
            surface,
            swapchain,
            images,
            event_loop: Some(eventloop),
        }
    }

    pub fn run(&mut self, event_sender: Sender<Arc<ROT_Event>>) {}

    pub fn get_window(&self) -> &Window {
        self.surface.window()
    }

    fn create_instance(required_extensions: &InstanceExtensions) -> Arc<Instance> {
        let instance_result = Instance::new(None, required_extensions, None);

        let instance = match instance_result {
            Ok(T) => {
                trace!("Instance, create successfully!");
                Some(T)
            }
            Err(err) => {
                error!("{}", err.to_string());
                None
            }
        };

        instance.unwrap()
    }

    fn get_physical_device(instance: &Arc<Instance>) -> PhysicalDevice {
        let physical_iter = PhysicalDevice::enumerate(instance);
        for (index, physical) in physical_iter.enumerate() {
            trace!("Device Availiable = {} - {}", index, physical.name());
        }
        trace!("Getting device 0");

        let physical = match PhysicalDevice::from_index(instance, 0) {
            Some(T) => {
                trace!("Physical Device Acquired Successfully");
                Some(T)
            }
            None => {
                error!("Failed to Acquire physical device");
                None
            }
        };

        ROT_Renderer::check_families(&physical.unwrap());
        physical.unwrap()
    }

    fn check_families(physical_device: &PhysicalDevice) {
        for family in physical_device.queue_families() {
            trace!(
                "Queue id {} supports {} queues. Support Graphics {} - Support_Compute {}",
                family.id(),
                family.queues_count(),
                family.supports_graphics(),
                family.supports_compute()
            )
        }
    }

    fn get_queue_family<'a>(
        physical_device: &'a PhysicalDevice,
        surface: &'a Arc<Surface<Window>>,
    ) -> QueueFamily<'a> {
        let queue_family = match physical_device
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        {
            Some(queue) => {
                trace!(
                    "Acquired queue family with id {} for device creation",
                    queue.id()
                );
                Some(queue)
            }
            None => {
                trace!("Failed to aquire a graphics queue family");
                None
            }
        };

        queue_family.unwrap()
    }

    fn create_surface(
        instance: &Arc<Instance>,
        dimensions: [u32; 2],
    ) -> (Arc<Surface<Window>>, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .with_min_inner_size(winit::dpi::LogicalSize::new(
                dimensions[0] as f64,
                dimensions[1] as f64,
            ))
            .with_resizable(false)
            .build_vk_surface(&event_loop, instance.clone());

        match &surface {
            Ok(_) => {
                trace!("Surface acquired on window",)
            }
            Err(err) => {
                error!("failed to acquired surface. Error = {}", err)
            }
        };

        (surface.unwrap(), event_loop)
    }

    fn get_device_and_queueiter(
        phys_device: &PhysicalDevice,
        queue_family: &QueueFamily,
    ) -> (Arc<Device>, QueuesIter) {
        let rqd_device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        let result = Device::new(
            *phys_device,
            phys_device.supported_features(),
            &rqd_device_ext,
            [(*queue_family, 0.5)].iter().cloned(),
        );
        match result {
            Ok(_) => {
                trace!("Device creation succesfull")
            }

            Err(err) => {
                error!("Failed to create device. Error {}", err)
            }
        };

        result.unwrap()
    }

    fn create_swapchain(
        physical_device: &PhysicalDevice,
        surface: &Arc<Surface<Window>>,
        device: &Arc<Device>,
        queue: &Arc<Queue>,
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        let surface_capabilities = surface.capabilities(*physical_device).unwrap();
        let alpha_behaviour = surface_capabilities
            .supported_composite_alpha
            .iter()
            .next()
            .unwrap();
        let image_format = surface_capabilities.supported_formats[0].0;
        let image_dimension = surface.window().inner_size().into();

        let swapchain_result = Swapchain::new(
            device.clone(),
            surface.clone(),
            surface_capabilities.min_image_count,
            image_format,
            image_dimension,
            1,
            ImageUsage::color_attachment(),
            queue,
            SurfaceTransform::Identity,
            alpha_behaviour,
            PresentMode::Immediate,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        );

        match &swapchain_result {
            Ok(_) => {
                trace!("Swapchain created successful")
            }
            Err(err) => {
                error!("Failed to create swapchain error = {}", err)
            }
        }

        swapchain_result.unwrap()
    }

    pub fn take_eventloop(&mut self) -> EventLoop<()> {
        self.event_loop.take().unwrap()
    }

    pub fn adjust_framebuffers(
        images: &[Arc<SwapchainImage<Window>>],
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dynamic_state: &mut DynamicState,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        // pegando as dimensões das imagens. Todas devem ser igual
        let dimensions = images[0].dimensions();

        // criando uma viewport nova com essas dimensões...
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        // .. e colocando ela no dinamic_state
        dynamic_state.viewports = Some(vec![viewport]);

        images
            .iter()
            //em cada imagem ..
            .map(|image| {
                // criar um framebuffer
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            // então colocá--los em um vector e retorna-lo
            .collect::<Vec<_>>()
    }
}
