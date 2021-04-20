#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, Device, Entry, Instance};

use std::collections::HashMap;
use std::ffi::{CStr, CString};

use winit::dpi::PhysicalSize;
use winit::dpi::Size::Physical;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod graphics_pipeline;
use graphics_pipeline::GraphicsPipeline;

mod sync_usage;
use sync_usage::{FenceUsage, SemaphoreUsage};

mod common;
use common::Vertex;
use std::mem::size_of;

pub struct Renderer {
    //Sync
    fences_map: HashMap<FenceUsage, usize>,
    fences_vec: Vec<vk::Fence>,
    semaphore_map: HashMap<SemaphoreUsage, usize>,
    semaphores_vec: Vec<vk::Semaphore>,

    //Buffers
    //memory_requirements: MemoryRequirements,
    command_buffer: Vec<vk::CommandBuffer>,
    vertex_buffer: vk::Buffer,

    //Pools
    command_pool: vk::CommandPool,

    //Framebuffer
    swapchain_framebuffer: Vec<vk::Framebuffer>,

    //Graphics Pipeline
    graphics_pipeline: GraphicsPipeline,

    //Swapchain
    swpch_format: vk::SurfaceFormatKHR,
    swpch_present_mode: vk::PresentModeKHR,
    swpch_extent: vk::Extent2D,
    swpch_imageview: Vec<vk::ImageView>,
    swpch_images: Vec<vk::Image>,
    swapchain: vk::SwapchainKHR,
    swapchain_loader: Swapchain,

    //Device
    phys_device: vk::PhysicalDevice,
    device: Device,

    //Queue
    graphics_queue: vk::Queue,
    queue_family: u32,

    //Debug
    is_debug: bool,
    debug_manager: Option<vk::DebugUtilsMessengerEXT>,
    debug_loader: Option<DebugUtils>,

    //Instance and Surface
    surface: vk::SurfaceKHR,
    surface_loader: Surface,
    instance_loader: Instance,
    entry_loader: Entry,

    //Window
    window: Window,
    event_loop: Option<EventLoop<()>>,
    minimized: bool,

    //Frame in flight
    max_frames_in_flight: u8,
    current_frame: usize,
}
impl Renderer {
    pub fn build(dimension: [u32; 2], debug: bool, frames_in_flight: u8) -> Self {
        let device_extension_needed: Vec<*const i8> = vec![Swapchain::name().as_ptr()];
        let layer = CString::new("VK_LAYER_KHRONOS_validation").unwrap();

        let device_layers_needed: Vec<*const i8> = if debug {
            vec![layer.as_ptr()]
        } else {
            Vec::new()
        };

        info!("SETTING UP VULKAN RENDERER");

        trace!("Building Event loop and window");
        let (event_loop, window) = Renderer::window_init(dimension);

        trace!("Loading Entry Functions");
        let entry_loader = unsafe { Entry::new() }.unwrap();
        trace!(
            "{} - Vulkan Instance {}.{}.{}",
            "Loaded!",
            vk::version_major(
                entry_loader
                    .try_enumerate_instance_version()
                    .unwrap()
                    .unwrap()
            ),
            vk::version_minor(
                entry_loader
                    .try_enumerate_instance_version()
                    .unwrap()
                    .unwrap()
            ),
            vk::version_patch(
                entry_loader
                    .try_enumerate_instance_version()
                    .unwrap()
                    .unwrap()
            )
        );

        trace!("Building Instance");
        let instance_loader = Renderer::create_instance(&entry_loader, &window, debug);

        trace!("Getting Surface and setting it up");
        let (surface_loader, surface) =
            Renderer::create_surface(&entry_loader, &instance_loader, &window);

        let mut debug_loader = None;
        let mut debug_manager = None;
        if debug {
            trace!("Building Vulkan Debug callback");
            let (loader, manager) = Renderer::build_debug_callback(&entry_loader, &instance_loader);

            debug_loader = Some(loader);
            debug_manager = Some(manager);
        }

        trace!("Querying Physical Device and it's graphics queues");
        let (phys_device, queue_family) =
            Renderer::get_physical_device(&instance_loader, &surface_loader, &surface).unwrap();

        trace!("Getting format, present mode and physical device properties");
        let (format, present_mode, device_properties) = Renderer::get_physical_device_data(
            &instance_loader,
            &phys_device,
            &surface_loader,
            &surface,
        )
        .unwrap();

        trace!("Creating device and queue");
        let (device, graphics_queue) = Renderer::create_device_and_queue(
            &instance_loader,
            &phys_device,
            queue_family,
            &device_extension_needed,
            &device_layers_needed,
        );

        trace!("Building the Swapchain");
        let (swapchain_loader, swapchain, images_vec, extend) =
            Renderer::build_swapchain_and_images(
                &instance_loader,
                &device,
                &phys_device,
                &surface_loader,
                &surface,
                &format,
                &present_mode,
            );

        trace!("images vec len {}", images_vec.len());

        trace!("Creating Images View");
        let imageview_vec = Renderer::create_images_view(&device, &images_vec, &format);

        info!("VULKAN RENDERED BUILT!");

        info!("BUILDING GRAPHICS PIPELINE");
        let graphics_pipeline = GraphicsPipeline::build(&device, &extend, &format);
        info!("GRAPHICS PIPELINE BUILT");

        trace!("Creating Framebuffer");
        let swapchain_framebuffer = Renderer::create_framebuffer(
            &device,
            &imageview_vec,
            graphics_pipeline.render_pass,
            &extend,
        );

        trace!("Building Command Pools");
        let command_pool = Renderer::create_command_pool(&device, queue_family);

        trace!("Building Vertex Buffer");
        let vertex_buffer = Renderer::create_vertex_buffer(&device, &instance_loader, &phys_device);

        trace!("Building Command Buffer");
        let command_buffer =
            Renderer::create_command_buffer(&device, command_pool, images_vec.len());

        trace!("Creating Vulkan Sync Devices");
        let semaphores_vec = Renderer::create_semaphores(&device, frames_in_flight);
        let mut semaphore_map: HashMap<SemaphoreUsage, usize> = HashMap::new();
        let mut semaphore_vec_index: usize = 0;
        for index in 0..frames_in_flight as usize {
            semaphore_map.insert(SemaphoreUsage::ImageAvailable(index), semaphore_vec_index);
            semaphore_vec_index += 1;
            semaphore_map.insert(SemaphoreUsage::RenderFinished(index), semaphore_vec_index);
            semaphore_vec_index += 1;
        }

        let mut fences_vec = Renderer::create_fences(&device, frames_in_flight as u8);
        let mut fences_map: HashMap<FenceUsage, usize> = HashMap::new();
        let mut fence_vec_index: usize = 0;
        for index in 0..frames_in_flight as usize {
            fences_map.insert(FenceUsage::CommandBufferExec(index), fence_vec_index);
            fence_vec_index += 1;
        }
        for index in 0..images_vec.len() {
            fences_vec.push(vk::Fence::null());
            fences_map.insert(FenceUsage::ImageAvailable(index), fence_vec_index);
            fence_vec_index += 1;
        }

        info!("DONE!");
        Renderer {
            fences_map,
            fences_vec,
            semaphore_map,
            semaphores_vec,
            vertex_buffer,
            command_buffer,
            command_pool,
            swapchain_framebuffer,
            graphics_pipeline,
            window,
            event_loop,
            entry_loader,
            instance_loader,
            surface_loader,
            surface,
            queue_family,
            phys_device,
            device,
            graphics_queue,
            swapchain,
            swapchain_loader,
            swpch_images: images_vec,
            swpch_imageview: imageview_vec,
            swpch_present_mode: present_mode,
            swpch_format: format,
            swpch_extent: extend,
            is_debug: debug,
            debug_manager,
            debug_loader,
            max_frames_in_flight: frames_in_flight,
            current_frame: 0,
            minimized: false,
        }
    }

    pub fn run(&mut self) {
        self.record_commands();

        let mut event_loop = self.event_loop.take().unwrap();

        event_loop.run_return(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    trace! {"New Window Size {:?}", size}
                    if size.height == 0 || size.width == 0 {
                        self.minimized = true;
                    } else {
                        self.minimized = false;
                    }
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    let vkc = input.virtual_keycode;
                    match vkc {
                        Some(key) => match key {
                            event::VirtualKeyCode::A => {
                                trace!("{:?}", self.fences_vec)
                            }
                            _ => {}
                        },
                        None => {}
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                self.draw_frame();
            }
            _ => {}
        })
    }

    pub fn destroy(&mut self) {
        warn!("Destroying!!");

        unsafe {
            self.destroy_swapchain();

            self.device.destroy_buffer(self.vertex_buffer, None);

            for index in 0..self.max_frames_in_flight {
                let fence = self.fences_vec[index as usize];
                self.device.destroy_fence(fence, None);
                trace!("Fence {} done", index)
            }

            for (index, &semaphore) in self.semaphores_vec.iter().enumerate() {
                self.device.destroy_semaphore(semaphore, None);
                trace!("Semaphore {} done", index)
            }

            self.device.destroy_command_pool(self.command_pool, None);
            trace!("Command Pool done");

            self.device.destroy_device(None);
            trace!("Device done");

            if self.debug_manager != None {
                self.debug_loader
                    .as_mut()
                    .unwrap()
                    .destroy_debug_utils_messenger(self.debug_manager.take().unwrap(), None);
                trace!("Debug done");
            }

            self.surface_loader.destroy_surface(self.surface, None);
            trace!("Surface done");

            self.instance_loader.destroy_instance(None);
            trace!("Instance done");
        }
        warn!("All destroyed!")
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }
}

impl Renderer {
    fn draw_frame(&mut self) {
        //get scoped variables for ease use
        let current_frame = self.current_frame;

        let command_buffer_fence = self.fences_vec[*self
            .fences_map
            .get(&FenceUsage::CommandBufferExec(current_frame))
            .unwrap()];

        let image_available_semaphore = self.semaphores_vec[*self
            .semaphore_map
            .get(&SemaphoreUsage::ImageAvailable(current_frame))
            .unwrap()];

        //Wait for the command buffer related to this frame be available
        unsafe {
            self.device
                .wait_for_fences(&[command_buffer_fence], true, u64::MAX)
                .unwrap();
        }

        //Get next image to render this frame on
        let image_index;
        unsafe {
            let result = self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                image_available_semaphore,
                vk::Fence::null(),
            );

            image_index = match result {
                Ok((index, _)) => index,
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        self.recreate_swapchain();
                        return;
                    } else {
                        panic!("Unknown error in acquire next image");
                    }
                }
            }
        }

        //Wait for the image to be available to render on

        let image_available_fence = self.fences_vec[*self
            .fences_map
            .get(&FenceUsage::ImageAvailable(image_index as usize))
            .unwrap()];
        if image_available_fence != vk::Fence::null() {
            unsafe {
                self.device
                    .wait_for_fences(&[image_available_fence], true, u64::MAX)
                    .unwrap();
            }
        }

        //Mark the image as now being in use by this frame, making the fence the same;
        self.fences_vec[*self
            .fences_map
            .get(&FenceUsage::ImageAvailable(image_index as usize))
            .unwrap()] = command_buffer_fence;

        let wait_semaphores = vec![
            self.semaphores_vec[*self
                .semaphore_map
                .get(&SemaphoreUsage::ImageAvailable(current_frame))
                .unwrap()],
        ];
        let command_buffer = vec![self.command_buffer[image_index as usize]];
        let signal_semaphores = vec![
            self.semaphores_vec[*self
                .semaphore_map
                .get(&SemaphoreUsage::RenderFinished(current_frame))
                .unwrap()],
        ];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffer)
            .signal_semaphores(&signal_semaphores)
            .build();

        unsafe {
            self.device.reset_fences(&[command_buffer_fence]).unwrap();

            self.device
                .queue_submit(self.graphics_queue, &[submit_info], command_buffer_fence)
                .unwrap();
        }

        let swapchains = vec![self.swapchain];
        let image_indices = vec![image_index];

        let presente_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices)
            .build();

        unsafe {
            let result = self
                .swapchain_loader
                .queue_present(self.graphics_queue, &presente_info);

            match result {
                Ok(suboptimal) => {
                    if suboptimal {
                        self.recreate_swapchain()
                    }
                }
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        self.recreate_swapchain()
                    } else {
                        panic!("Unknown error in queue present");
                    }
                }
            }

            self.device.queue_wait_idle(self.graphics_queue).unwrap();
        }

        self.current_frame = (current_frame + 1) % self.max_frames_in_flight as usize;
    }

    fn recreate_swapchain(&mut self) {
        if self.minimized {
            return;
        }

        unsafe { self.device.device_wait_idle().unwrap() }

        self.destroy_swapchain();

        let (swapchain_loader, swapchain, images_vec, extent) =
            Renderer::build_swapchain_and_images(
                &self.instance_loader,
                &self.device,
                &self.phys_device,
                &self.surface_loader,
                &self.surface,
                &self.swpch_format,
                &self.swpch_present_mode,
            );
        let image_view_vec =
            Renderer::create_images_view(&self.device, &images_vec, &self.swpch_format);
        let graphics_pipeline = GraphicsPipeline::build(&self.device, &extent, &self.swpch_format);
        let swapchain_framebuffer = Renderer::create_framebuffer(
            &self.device,
            &image_view_vec,
            graphics_pipeline.render_pass,
            &extent,
        );
        let command_buffer =
            Renderer::create_command_buffer(&self.device, self.command_pool, images_vec.len());

        self.swapchain_loader = swapchain_loader;
        self.swapchain = swapchain;
        self.swpch_images = images_vec;
        self.swpch_extent = extent;
        self.swpch_imageview = image_view_vec;
        self.graphics_pipeline = graphics_pipeline;
        self.swapchain_framebuffer = swapchain_framebuffer;
        self.command_buffer = command_buffer;

        self.record_commands();
    }

    fn destroy_swapchain(&mut self) {
        unsafe {
            //Waiting for idle
            self.device.device_wait_idle().unwrap();

            for (index, &framebuffer) in self.swapchain_framebuffer.iter().enumerate() {
                self.device.destroy_framebuffer(framebuffer, None);
                trace!("Framebuffer {} done", index)
            }

            self.device
                .free_command_buffers(self.command_pool, &self.command_buffer);
            trace!("Command buffer freed");

            self.graphics_pipeline.destroy(&self.device);
            trace!("Graphics pipeline done");

            for (index, &image_view) in self.swpch_imageview.iter().enumerate() {
                self.device.destroy_image_view(image_view, None);
                trace!("ImageView {} done", index)
            }

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            trace!("Device done");
        }
    }

    fn create_fences(device: &Device, count: u8) -> Vec<vk::Fence> {
        let mut fences_vec = Vec::new();
        for _x in 0..count {
            let fence_info = vk::FenceCreateInfo::builder()
                .flags(vk::FenceCreateFlags::SIGNALED)
                .build();
            let fence = unsafe { device.create_fence(&fence_info, None) }.unwrap();
            fences_vec.push(fence);
        }

        fences_vec
    }

    fn create_semaphores(device: &Device, count: u8) -> Vec<vk::Semaphore> {
        let mut semaphores = Vec::new();

        for _x in 0..(2 * count) {
            let semaphore_info = vk::SemaphoreCreateInfo::builder().build();
            let semaphore = unsafe { device.create_semaphore(&semaphore_info, None) }.unwrap();

            semaphores.push(semaphore)
        }

        semaphores
    }

    fn record_commands(&mut self) {
        for (&command_buffer, &framebuffer) in self
            .command_buffer
            .iter()
            .zip(self.swapchain_framebuffer.iter())
        {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder().build();
            unsafe {
                self.device
                    .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            }
            .unwrap();

            let clear_values = vec![vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }];

            let render_area = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swpch_extent,
            };

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.graphics_pipeline.render_pass)
                .framebuffer(framebuffer)
                .render_area(render_area)
                .clear_values(&clear_values)
                .build();

            unsafe {
                self.device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );

                self.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.graphics_pipeline.pipeline,
                );

                self.device.cmd_draw(command_buffer, 3, 1, 0, 0);

                self.device.cmd_end_render_pass(command_buffer);

                self.device.end_command_buffer(command_buffer).unwrap();
            }
        }
    }

    fn create_command_buffer(
        device: &Device,
        command_pool: vk::CommandPool,
        count: usize,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count as u32)
            .build();

        let command_buffer =
            unsafe { device.allocate_command_buffers(&command_buffer_alloc_info) }.unwrap();

        command_buffer
    }

    fn get_memory_id(
        memory_type_bit: u32,
        phys_device_memory_types: [vk::MemoryType; 32],
        phys_device_memory_type_count: u32,
        properties_required: vk::MemoryPropertyFlags,
    ) -> u32 {
        for x in 0..phys_device_memory_type_count {
            if memory_type_bit & (1 << x) != 0
                && phys_device_memory_types[x as usize].property_flags & properties_required
                    == properties_required
            {
                return x;
            }
        }
        panic!("No memory compatible");
    }

    fn create_vertex_buffer(
        device: &Device,
        instance_loader: &Instance,
        phys_device: &vk::PhysicalDevice,
    ) -> vk::Buffer {
        let size = size_of::<Vertex>() * Vertex::get_exemple_vector().len();

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe { device.create_buffer(&buffer_info, None) }.unwrap();

        let memory_id;
        unsafe {
            let buffer_memory_requirement = device.get_buffer_memory_requirements(buffer);

            let phys_device_memory_properties =
                instance_loader.get_physical_device_memory_properties(*phys_device);

            memory_id = Renderer::get_memory_id(
                buffer_memory_requirement.memory_type_bits,
                phys_device_memory_properties.memory_types,
                phys_device_memory_properties.memory_type_count,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );
        }

        buffer
    }

    fn create_command_pool(device: &Device, queue_family: u32) -> vk::CommandPool {
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family)
            .build();

        let command_pool = unsafe { device.create_command_pool(&pool_info, None) }.unwrap();

        command_pool
    }

    fn create_framebuffer(
        device: &Device,
        image_view_vec: &[vk::ImageView],
        render_pass: vk::RenderPass,
        extent: &vk::Extent2D,
    ) -> Vec<vk::Framebuffer> {
        let framebuffer: Vec<_> = image_view_vec
            .iter()
            .map(|image_view| {
                let attachment = vec![*image_view];
                let framebuffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachment)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1)
                    .build();

                unsafe { device.create_framebuffer(&framebuffer_info, None) }.unwrap()
            })
            .collect();

        framebuffer
    }

    fn create_images_view(
        device: &Device,
        images: &[vk::Image],
        format: &vk::SurfaceFormatKHR,
    ) -> Vec<vk::ImageView> {
        let swapchain_image_views = images
            .iter()
            .map(|image| {
                let image_view_info = vk::ImageViewCreateInfo::builder()
                    .image(*image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build(),
                    );
                unsafe { device.create_image_view(&image_view_info, None) }.unwrap()
            })
            .collect::<Vec<vk::ImageView>>();

        swapchain_image_views
    }

    fn build_swapchain_and_images(
        instance_loader: &Instance,
        device: &Device,
        phys_device: &vk::PhysicalDevice,
        surface_loader: &Surface,
        surface: &vk::SurfaceKHR,
        format: &vk::SurfaceFormatKHR,
        present_mode: &vk::PresentModeKHR,
    ) -> (Swapchain, vk::SwapchainKHR, Vec<vk::Image>, vk::Extent2D) {
        //Getting number os images for the swapchain from the surface
        let surface_caps = unsafe {
            surface_loader.get_physical_device_surface_capabilities(*phys_device, *surface)
        }
        .unwrap();
        let mut image_count = surface_caps.min_image_count + 1;
        if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
            image_count = surface_caps.max_image_count;
        }

        //building swapchain
        let swapchain_loader = Swapchain::new(instance_loader, device);

        let swapchain_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(surface_caps.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(*present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain =
            unsafe { swapchain_loader.create_swapchain(&swapchain_info, None) }.unwrap();
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }.unwrap();

        (
            swapchain_loader,
            swapchain,
            swapchain_images,
            surface_caps.current_extent,
        )
    }

    fn create_device_and_queue(
        instance_loader: &Instance,
        phys_device: &vk::PhysicalDevice,
        queue_family_index: u32,
        device_extensions: &[*const i8],
        device_layers: &[*const i8],
    ) -> (Device, vk::Queue) {
        let queue_info = vec![vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0])
            .build()];

        let phys_device_features = vk::PhysicalDeviceFeatures::builder();

        let device_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_info)
            .enabled_features(&phys_device_features)
            .enabled_extension_names(device_extensions)
            .enabled_layer_names(device_layers);

        let device =
            unsafe { instance_loader.create_device(*phys_device, &device_info, None) }.unwrap();
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        (device, queue)
    }

    fn get_physical_device_data(
        instance_loader: &Instance,
        phys_device: &vk::PhysicalDevice,
        surface_loader: &Surface,
        surface: &vk::SurfaceKHR,
    ) -> (
        vk::SurfaceFormatKHR,
        vk::PresentModeKHR,
        vk::PhysicalDeviceProperties,
    ) {
        unsafe {
            let mut format = None;
            let mut present_mode = None;
            let mut device_properties = None;

            //Quarrying format
            {
                let format_vec = surface_loader
                    .get_physical_device_surface_formats(*phys_device, *surface)
                    .unwrap();

                for surface_format in &format_vec {
                    if (surface_format.format == vk::Format::R8G8B8A8_SRGB)
                        && (surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
                    {
                        format = Some(*surface_format);
                        break;
                    }
                }
                match format {
                    None => format = Some(format_vec[0]),
                    Some(_) => {}
                }
            }

            //Quarrying present mode
            {
                let present_mode_vec = surface_loader
                    .get_physical_device_surface_present_modes(*phys_device, *surface)
                    .unwrap();

                for prsnt_mode in present_mode_vec {
                    if prsnt_mode == vk::PresentModeKHR::MAILBOX {
                        present_mode = Some(prsnt_mode);
                        break;
                    }
                }
                if present_mode == None {
                    present_mode = Some(vk::PresentModeKHR::FIFO)
                }
            }

            //Quarrying device properties
            {
                let device_extensions_needed = vec![Swapchain::name().as_ptr()];
                let supported_device_extensions = instance_loader
                    .enumerate_device_extension_properties(*phys_device)
                    .unwrap();

                let is_all_device_ext_supported =
                    device_extensions_needed.iter().all(|device_extension| {
                        let device_extension_name = CStr::from_ptr(*device_extension);

                        supported_device_extensions.iter().any(|properties| {
                            CStr::from_ptr(properties.extension_name.as_ptr())
                                == device_extension_name
                        })
                    });

                if !is_all_device_ext_supported {
                    error!("Not all needed device extension is available");
                }
                device_properties =
                    Some(instance_loader.get_physical_device_properties(*phys_device));
            }

            (
                format.unwrap(),
                present_mode.unwrap(),
                device_properties.unwrap(),
            )
        }
    }

    fn get_physical_device(
        instance_loader: &Instance,
        surface_loader: &Surface,
        surface_khr: &vk::SurfaceKHR,
    ) -> Option<(vk::PhysicalDevice, u32)> {
        unsafe {
            let phys_device_vec = instance_loader.enumerate_physical_devices().unwrap();

            for phys_device in phys_device_vec {
                let queue_family_properties_vec =
                    instance_loader.get_physical_device_queue_family_properties(phys_device);

                for (queue_family, queue_family_properties) in
                    queue_family_properties_vec.into_iter().enumerate()
                {
                    let queue_flags_valid = queue_family_properties
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS);

                    let device_surface_support = surface_loader
                        .get_physical_device_surface_support(
                            phys_device,
                            queue_family as u32,
                            *surface_khr,
                        )
                        .unwrap();

                    if queue_flags_valid && device_surface_support {
                        return Some((phys_device, queue_family as u32));
                    }
                }
            }

            None
        }
    }

    fn create_surface(
        entry_loader: &Entry,
        instance_loader: &Instance,
        window: &Window,
    ) -> (Surface, vk::SurfaceKHR) {
        let surface_loader = Surface::new(entry_loader, instance_loader);
        let surface = unsafe {
            ash_window::create_surface(entry_loader, instance_loader, window, None).unwrap()
        };

        (surface_loader, surface)
    }

    fn window_init(dimensions: [u32; 2]) -> (Option<EventLoop<()>>, Window) {
        let eventloop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_resizable(true)
            .with_inner_size(Physical(PhysicalSize::new(dimensions[0], dimensions[1])))
            .with_title("Vulkan Rendering for ROT_Engine")
            .build(&eventloop)
            .unwrap();

        (Some(eventloop), window)
    }

    fn create_instance(entry: &Entry, window: &Window, debug: bool) -> Instance {
        let app_name = CString::new("ROT_Builder").unwrap();
        let eng_name = CString::new("ROT_Engine").unwrap();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_version(0, 1, 0))
            .engine_version(vk::make_version(0, 1, 0))
            .engine_name(&eng_name)
            .api_version(vk::make_version(1, 0, 0));

        let surface_extension = ash_window::enumerate_required_extensions(window).unwrap();
        let mut instance_extension_raw = surface_extension
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let mut instance_layer_raw = Vec::new();
        let instance_layer = vec![CString::new("VK_LAYER_KHRONOS_validation").unwrap()];

        if debug {
            instance_extension_raw.push(DebugUtils::name().as_ptr());

            instance_layer_raw = instance_layer.iter().map(|layer| layer.as_ptr()).collect();
        }

        let instance_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&instance_layer_raw)
            .enabled_extension_names(&instance_extension_raw)
            .build();

        unsafe { entry.create_instance(&instance_info, None) }.unwrap()
    }

    fn build_debug_callback(
        entry_loader: &Entry,
        instance: &Instance,
    ) -> (DebugUtils, vk::DebugUtilsMessengerEXT) {
        let messenger_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(debug_callback));

        let debug_loader = DebugUtils::new(entry_loader, instance);
        let debug_callback =
            unsafe { debug_loader.create_debug_utils_messenger(&messenger_info, None) }.unwrap();

        (debug_loader, debug_callback)
    }
}

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            /*
            trace!(
                "{}",
                CStr::from_ptr((*p_callback_data).p_message).to_string_lossy()
            );
            */
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            info!(
                "Type = {:?} -> {}",
                message_type,
                CStr::from_ptr((*p_callback_data).p_message).to_string_lossy()
            );
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            warn!(
                "Type = {:?} -> {}",
                message_type,
                CStr::from_ptr((*p_callback_data).p_message).to_string_lossy()
            );
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            error!(
                "Type = {:?} -> {}",
                message_type,
                CStr::from_ptr((*p_callback_data).p_message).to_string_lossy()
            );
        }

        _ => {}
    }

    vk::FALSE
}
