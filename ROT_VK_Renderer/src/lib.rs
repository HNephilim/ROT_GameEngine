use log::{debug, error, info, trace, warn};

use std::borrow::Borrow;
use std::iter::Cloned;
use std::sync::Arc;

mod graphics_structs;
use graphics_structs::Vertex2D;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::{Device, DeviceExtensions, Features, Queue, QueuesIter};
use vulkano::framebuffer::Framebuffer;
use vulkano::image;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice, QueueFamily};
use vulkano::pipeline::GraphicsPipeline;

use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform,
    Swapchain, SwapchainCreationError,
};
use vulkano::sync;

use vulkano_win::{CreationError, VkSurfaceBuild};

use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState, SubpassContents};
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::pipeline::viewport::Viewport;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_shaders;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};

pub struct ROT_Renderer {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,

    event_loop: Option<EventLoop<()>>,
    triangle: Option<[Vertex2D; 3]>,
}

impl ROT_Renderer {
    pub fn build() -> Self {
        info!("VULKAN RENDERER BUILDING!");
        let rqd_instance_ext = vulkano_win::required_extensions();
        let instance = ROT_Renderer::create_instance(&rqd_instance_ext);

        let physical_device = ROT_Renderer::get_physical_device(&instance);

        let surface_and_eventloop = ROT_Renderer::create_surface(&instance);
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
            triangle: None,
        }
    }

    pub fn run(&mut self) {
        let mut triangle = [
            Vertex2D {
                position: [-0.5, -0.25],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex2D {
                position: [0.0, 0.5],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex2D {
                position: [0.25, -0.1],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            false,
            triangle.iter().cloned(),
        )
        .unwrap();

        mod vs {
            vulkano_shaders::shader! {
                ty: "vertex",
                src: "
				#version 450

				layout(location = 0) in vec2 position;

				void main() {
					gl_Position = vec4(position, 0.0, 1.0);
				}
			"
            }
        }

        mod fs {
            vulkano_shaders::shader! {
                ty: "fragment",
                src: "
				#version 450

				layout(location = 0) out vec4 f_color;

				void main() {
					f_color = vec4(1.0, 0.0, 0.0, 1.0);
				}
			"
            }
        }

        let vs = vs::Shader::load(self.device.clone()).unwrap();
        let fs = fs::Shader::load(self.device.clone()).unwrap();

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments:{
                color: {
                    load: Clear,
                    store: Store,
                    format: self.swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
            )
            .unwrap(),
        );

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(self.device.clone())
                .unwrap(),
        );

        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };

        let mut framebuffers =
            window_size_dependent_setup(&self.images, render_pass.clone(), &mut dynamic_state);

        let mut recreate_swapchain = false;
        let mut previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        let mut event_loop = self.event_loop.take().unwrap();
        let surface = self.surface.clone();
        let mut swapchain = self.swapchain.clone();
        let device = self.device.clone();
        let queue = self.queue.clone();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event: ev, .. } => match ev {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(_) => {
                        recreate_swapchain = true;
                    }
                    WindowEvent::KeyboardInput { input, .. } => {}

                    _ => {}
                },
                Event::RedrawEventsCleared => {
                    previous_frame_end.as_mut().unwrap().cleanup_finished();

                    if recreate_swapchain {
                        let dimensions: [u32; 2] = surface.window().inner_size().into();
                        let (new_swapchain, new_images) =
                            match swapchain.recreate_with_dimensions(dimensions) {
                                Ok(r) => r,
                                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                            };

                        swapchain = new_swapchain;

                        framebuffers = window_size_dependent_setup(
                            &new_images,
                            render_pass.clone(),
                            &mut dynamic_state,
                        );
                        recreate_swapchain = false;
                    }

                    let (image_num, suboptimal, acquire_future) =
                        match swapchain::acquire_next_image(swapchain.clone(), None) {
                            Ok(r) => r,
                            Err(AcquireError::OutOfDate) => {
                                recreate_swapchain = true;
                                return;
                            }
                            Err(e) => panic!("Failed to acquire next image: {:?}", e),
                        };

                    if suboptimal {
                        recreate_swapchain = true;
                    }

                    let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];

                    let mut command_builder = AutoCommandBufferBuilder::primary_one_time_submit(
                        device.clone(),
                        queue.clone().family(),
                    )
                    .unwrap();

                    command_builder
                        .begin_render_pass(
                            framebuffers[image_num].clone(),
                            SubpassContents::Inline,
                            clear_values,
                        )
                        .unwrap()
                        .draw(
                            pipeline.clone(),
                            &dynamic_state,
                            vertex_buffer.clone(),
                            (),
                            (),
                            vec![],
                        )
                        .unwrap()
                        .end_render_pass()
                        .unwrap();

                    let command_buffer = command_builder.build().unwrap();

                    let future = previous_frame_end
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(queue.clone(), command_buffer)
                        .unwrap()
                        .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                        .then_signal_fence_and_flush();

                    match future {
                        Ok(future) => previous_frame_end = Some(future.boxed()),
                        Err(FlushError::OutOfDate) => {
                            recreate_swapchain = true;
                            previous_frame_end = Some(sync::now(device.clone()).boxed())
                        }
                        Err(e) => {
                            warn!("Failed to flush future: {:?}", e);
                            previous_frame_end = Some(sync::now(device.clone()).boxed())
                        }
                    }
                }
                _ => {}
            }
        });
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

    fn create_surface(instance: &Arc<Instance>) -> (Arc<Surface<Window>>, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone());

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
}

fn window_size_dependent_setup(
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
