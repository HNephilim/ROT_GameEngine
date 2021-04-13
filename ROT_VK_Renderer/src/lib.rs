#[allow(unused_imports)]
#[allow(non_camel_case_types)]
use log::{debug, error, info, trace, warn};

use erupt::extensions::ext_debug_utils::{
    DebugUtilsMessageSeverityFlagBitsEXT, DebugUtilsMessengerEXT,
};
use erupt::vk::{
    make_version, ApplicationInfoBuilder, CommandBuffer, CommandPool, Extent2D, Fence, Format,
    Framebuffer, Image, ImageView, Instance, PhysicalDevice, PhysicalDeviceProperties,
    PresentModeKHR, Queue, RenderPass, Semaphore, SurfaceFormatKHR, SurfaceKHR, SwapchainKHR,
};
use erupt::{
    cstr,
    utils::{self, surface},
    vk, DefaultEntryLoader, DeviceLoader, EntryLoader, InstanceLoader,
};
use std::ffi::{c_void, CStr, CString};
use winit::dpi::PhysicalSize;
use winit::dpi::Size::Physical;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;
use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod graphics_pipeline;
use graphics_pipeline::GraphicsPipeline;

use std::collections::HashMap;
mod sync_usage;
use erupt::utils::VulkanResult;
use std::hash::Hash;
use sync_usage::{FenceUsage, SemaphoreUsage};

pub struct ROT_Renderer {
    //Sync
    fences_map: HashMap<FenceUsage, usize>,
    fences_vec: Vec<Fence>,
    semaphore_map: HashMap<SemaphoreUsage, usize>,
    semaphores_vec: Vec<Semaphore>,

    //Command Buffer
    command_buffer: Vec<CommandBuffer>,

    //Pools
    command_pool: CommandPool,

    //Framebuffer
    swapchain_framebuffer: Vec<Framebuffer>,

    //Graphics Pipeline
    graphics_pipeline: GraphicsPipeline,

    //Swapchain
    swpch_format: SurfaceFormatKHR,
    swpch_present_mode: PresentModeKHR,
    swpch_extent: Extent2D,
    swpch_imageview: Vec<ImageView>,
    swpch_images: Vec<Image>,
    swapchain: SwapchainKHR,

    //Device
    phys_device: PhysicalDevice,
    device: DeviceLoader,
    //Queue
    graphics_queue: Queue,
    queue_family: u32,

    //Debug
    is_debug: bool,
    debug_manager: Option<DebugUtilsMessengerEXT>,

    //Instance and Surface
    surface: SurfaceKHR,
    instance: InstanceLoader,
    entry: DefaultEntryLoader,

    //Window
    window: Window,
    event_loop: Option<EventLoop<()>>,

    //Frame in flight
    max_frames_in_flight: u8,
    current_frame: usize,
}
impl ROT_Renderer {
    pub fn build(dimension: [u32; 2], debug: bool, frames_in_flight: u8) -> Self {
        let device_extension_needed: Vec<*const i8> = vec![vk::KHR_SWAPCHAIN_EXTENSION_NAME];
        let device_layers_needed: Vec<*const i8> = if debug {
            vec![cstr!("VK_LAYER_KHRONOS_validation")]
        } else {
            Vec::new()
        };

        info!("SETTING UP VULKAN RENDERER");

        trace!("Building Event loop and window");
        let (event_loop, window) = ROT_Renderer::window_init(dimension);

        trace!("Loading Entry Functions");
        let entry = EntryLoader::new().unwrap();
        trace!(
            "{} - Vulkan Instance {}.{}.{}",
            "Loaded!",
            vk::version_major(entry.instance_version()),
            vk::version_minor(entry.instance_version()),
            vk::version_patch(entry.instance_version())
        );

        trace!("Building Instance");
        let instance = ROT_Renderer::create_instance_loader(&entry, &window, debug);

        trace!("Getting Surface and setting it up");
        let surface = ROT_Renderer::create_surface(&instance, &window);

        let mut debug_manager = None;
        if debug {
            trace!("Building Vulkan Debug callback");
            debug_manager = Some(ROT_Renderer::build_debug_calback(&instance));
        }

        trace!("Querying Physical Device and it's graphics queues");
        let (phys_device, queue_family) =
            ROT_Renderer::get_physical_device(&instance, &surface).unwrap();

        trace!("Getting format, present mode and physical device properties");
        let (format, present_mode, device_properties) =
            ROT_Renderer::get_physical_device_data(&instance, &phys_device, &surface).unwrap();

        trace!("Creating device and queue");
        let (device, graphics_queue) = ROT_Renderer::create_device_and_queue(
            &instance,
            &phys_device,
            queue_family,
            &device_extension_needed,
            &device_layers_needed,
        );

        trace!("Building the Swapchain");
        let (swapchain, images_vec, extend) = ROT_Renderer::build_swapchain_and_images(
            &instance,
            &device,
            &phys_device,
            surface,
            &format,
            &present_mode,
        );

        trace!("images vec len {}", images_vec.len());

        trace!("Creating Images View");
        let imageview_vec = ROT_Renderer::create_images_view(&device, &images_vec, &format);

        info!("VULKAN RENDERED BUILT!");

        info!("BUILDING GRAPHICS PIPELINE");
        let graphics_pipeline = GraphicsPipeline::build(&device, &extend, &format);
        info!("GRAPHICS PIPELINE BUILT");

        trace!("Creating Framebuffer");
        let swapchain_framebuffer = ROT_Renderer::create_framebuffer(
            &device,
            &imageview_vec,
            graphics_pipeline.render_pass,
            &extend,
        );

        trace!("Building Command Pools");
        let command_pool = ROT_Renderer::create_command_pool(&device, queue_family);

        trace!("Building Command Buffer");
        let command_buffer =
            ROT_Renderer::create_command_buffer(&device, command_pool, images_vec.len());

        trace!("Creating Vulkan Sync Devices");
        let semaphores_vec = ROT_Renderer::create_semaphores(&device, frames_in_flight);
        let mut semaphore_map: HashMap<SemaphoreUsage, usize> = HashMap::new();
        let mut semaphore_vec_index: usize = 0;
        for index in 0..frames_in_flight as usize {
            semaphore_map.insert(SemaphoreUsage::ImageAvailable(index), semaphore_vec_index);
            semaphore_vec_index += 1;
            semaphore_map.insert(SemaphoreUsage::RenderFinished(index), semaphore_vec_index);
            semaphore_vec_index += 1;
        }

        let mut fences_vec = ROT_Renderer::create_fences(&device, (frames_in_flight as u8));
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
        ROT_Renderer {
            fences_map,
            fences_vec,
            semaphore_map,
            semaphores_vec,
            command_buffer,
            command_pool,
            swapchain_framebuffer,
            graphics_pipeline,
            window,
            event_loop,
            entry,
            instance,
            surface,
            queue_family,
            phys_device,
            device,
            graphics_queue,
            swapchain,
            swpch_images: images_vec,
            swpch_imageview: imageview_vec,
            swpch_present_mode: present_mode,
            swpch_format: format,
            swpch_extent: extend,
            is_debug: debug,
            debug_manager,
            max_frames_in_flight: frames_in_flight,
            current_frame: 0,
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
                WindowEvent::KeyboardInput { input, .. } => {
                    let vkc = input.virtual_keycode;
                    match vkc {
                        Some(key) => match key {
                            VirtualKeyCode::A => {
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

            for index in 0..self.max_frames_in_flight {
                let fence = self.fences_vec[index as usize];
                self.device.destroy_fence(Some(fence), None);
                trace!("Fence {} done", index)
            }

            for (index, &semaphore) in self.semaphores_vec.iter().enumerate() {
                self.device.destroy_semaphore(Some(semaphore), None);
                trace!("Semaphore {} done", index)
            }

            self.device
                .destroy_command_pool(Some(self.command_pool), None);
            trace!("Command Pool done");

            self.device.destroy_device(None);
            trace!("Device done");

            if self.debug_manager != None {
                self.instance
                    .destroy_debug_utils_messenger_ext(Some(self.debug_manager.unwrap()), None);

                trace!("Debug done");
            }

            self.instance.destroy_surface_khr(Some(self.surface), None);
            trace!("Surface done");

            self.instance.destroy_instance(None);
            trace!("Instance done");
        }
        warn!("All destroyed!")
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }
}

impl ROT_Renderer {
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
                .wait_for_fences(&[command_buffer_fence], true, u64::max_value())
                .unwrap();
        }

        //Get next image to render this frame on
        let image_index = unsafe {
            self.device.acquire_next_image_khr(
                self.swapchain,
                u64::max_value(),
                Some(image_available_semaphore),
                None,
                None,
            )
        }
        .unwrap();

        //Wait for the image to be available to render on

        let image_available_fence = self.fences_vec[*self
            .fences_map
            .get(&FenceUsage::ImageAvailable(image_index as usize))
            .unwrap()];
        if !image_available_fence.is_null() {
            unsafe {
                let result =
                    self.device
                        .wait_for_fences(&[image_available_fence], true, u64::max_value());

                match result {
                    VulkanResult { raw, value } => match value {
                        Some(_) => {}
                        None => match raw {
                            vk::Result::ERROR_OUT_OF_DATE_KHR => {
                                self.recreate_swapchain();
                            }
                            _ => {}
                        },
                    },
                }
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

        let submit_info = vk::SubmitInfoBuilder::new()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffer)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.device.reset_fences(&[command_buffer_fence]);

            self.device.queue_submit(
                self.graphics_queue,
                &[submit_info],
                Some(command_buffer_fence),
            )
        }
        .unwrap();

        let swapchains = vec![self.swapchain];
        let image_indices = vec![image_index];

        let presente_info = vk::PresentInfoKHRBuilder::new()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            let result = self
                .device
                .queue_present_khr(self.graphics_queue, &presente_info);

            match result {
                VulkanResult { raw, value } => match value {
                    Some(_) => {}
                    None => match raw {
                        vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => {
                            self.recreate_swapchain();
                        }
                        _ => {}
                    },
                },
            }

            self.device.queue_wait_idle(self.graphics_queue).unwrap();
        }

        self.current_frame = (current_frame + 1) % self.max_frames_in_flight as usize;
    }

    fn recreate_swapchain(&mut self) {
        unsafe { self.device.device_wait_idle().unwrap() }

        self.destroy_swapchain();

        let (swapchain, images_vec, extent) = ROT_Renderer::build_swapchain_and_images(
            &self.instance,
            &self.device,
            &self.phys_device,
            self.surface,
            &self.swpch_format,
            &self.swpch_present_mode,
        );
        let imageview_vec =
            ROT_Renderer::create_images_view(&self.device, &images_vec, &self.swpch_format);
        let graphics_pipeline = GraphicsPipeline::build(&self.device, &extent, &self.swpch_format);
        let swapchain_framebuffer = ROT_Renderer::create_framebuffer(
            &self.device,
            &imageview_vec,
            graphics_pipeline.render_pass,
            &extent,
        );
        let command_buffer =
            ROT_Renderer::create_command_buffer(&self.device, self.command_pool, images_vec.len());

        self.swapchain = swapchain;
        self.swpch_images = images_vec;
        self.swpch_extent = extent;
        self.swpch_imageview = imageview_vec;
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
                self.device.destroy_framebuffer(Some(framebuffer), None);
                trace!("Framebuffer {} done", index)
            }

            self.device
                .free_command_buffers(self.command_pool, &self.command_buffer);
            trace!("Command buffer freed");

            self.graphics_pipeline.destroy(&self.device);
            trace!("Graphics pipeline done");

            for (index, &image_view) in self.swpch_imageview.iter().enumerate() {
                self.device.destroy_image_view(Some(image_view), None);
                trace!("ImageView {} done", index)
            }

            self.device
                .destroy_swapchain_khr(Some(self.swapchain), None);
            trace!("Device done");
        }
    }

    fn create_fences(device: &DeviceLoader, count: u8) -> Vec<Fence> {
        let mut fences_vec = Vec::new();
        for _x in 0..count {
            let fence_info =
                vk::FenceCreateInfoBuilder::new().flags(vk::FenceCreateFlags::SIGNALED);
            let fence = unsafe { device.create_fence(&fence_info, None, None) }.unwrap();
            fences_vec.push(fence);
        }

        fences_vec
    }

    fn create_semaphores(device: &DeviceLoader, count: u8) -> Vec<Semaphore> {
        let mut semaphores = Vec::new();

        for _x in 0..(2 * count) {
            let semaphore_info = vk::SemaphoreCreateInfoBuilder::new();
            let semaphore =
                unsafe { device.create_semaphore(&semaphore_info, None, None) }.unwrap();

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
            let command_buffer_begin_info = vk::CommandBufferBeginInfoBuilder::new();
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

            let render_pass_begin_info = vk::RenderPassBeginInfoBuilder::new()
                .render_pass(self.graphics_pipeline.render_pass)
                .framebuffer(framebuffer)
                .render_area(render_area)
                .clear_values(&clear_values);

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
        device: &DeviceLoader,
        command_pool: CommandPool,
        count: usize,
    ) -> Vec<CommandBuffer> {
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count as u32);

        let command_buffer =
            unsafe { device.allocate_command_buffers(&command_buffer_alloc_info) }.unwrap();

        command_buffer
    }

    fn create_command_pool(device: &DeviceLoader, queue_family: u32) -> CommandPool {
        let pool_info = vk::CommandPoolCreateInfoBuilder::new().queue_family_index(queue_family);

        let command_pool = unsafe { device.create_command_pool(&pool_info, None, None) }.unwrap();

        command_pool
    }

    fn create_framebuffer(
        device: &DeviceLoader,
        image_view_vec: &Vec<ImageView>,
        render_pass: RenderPass,
        extent: &Extent2D,
    ) -> Vec<Framebuffer> {
        let framebuffer: Vec<_> = image_view_vec
            .iter()
            .map(|image_view| {
                let attachment = vec![*image_view];
                let framebuffer_info = vk::FramebufferCreateInfoBuilder::new()
                    .render_pass(render_pass)
                    .attachments(&attachment)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);

                unsafe { device.create_framebuffer(&framebuffer_info, None, None) }.unwrap()
            })
            .collect();

        framebuffer
    }

    fn create_images_view(
        device: &DeviceLoader,
        images: &Vec<Image>,
        format: &SurfaceFormatKHR,
    ) -> Vec<ImageView> {
        let swapchain_image_views = images
            .iter()
            .map(|image| {
                let image_view_info = vk::ImageViewCreateInfoBuilder::new()
                    .image(*image)
                    .view_type(vk::ImageViewType::_2D)
                    .format(format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(
                        vk::ImageSubresourceRangeBuilder::new()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build(),
                    );
                unsafe { device.create_image_view(&image_view_info, None, None) }.unwrap()
            })
            .collect::<Vec<ImageView>>();

        swapchain_image_views
    }

    fn build_swapchain_and_images(
        instance: &InstanceLoader,
        device: &DeviceLoader,
        phys_device: &PhysicalDevice,
        surface: SurfaceKHR,
        format: &SurfaceFormatKHR,
        present_mode: &PresentModeKHR,
    ) -> (SwapchainKHR, Vec<Image>, Extent2D) {
        //Getting number os images for the swapchain from the surface
        let surface_caps = unsafe {
            instance.get_physical_device_surface_capabilities_khr(*phys_device, surface, None)
        }
        .unwrap();
        let mut image_count = surface_caps.min_image_count + 1;
        if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
            image_count = surface_caps.max_image_count;
        }

        //building swapchain
        let swapchain_info = vk::SwapchainCreateInfoKHRBuilder::new()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(surface_caps.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagBitsKHR::OPAQUE_KHR)
            .present_mode(*present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain =
            unsafe { device.create_swapchain_khr(&swapchain_info, None, None) }.unwrap();
        let swapchain_images = unsafe { device.get_swapchain_images_khr(swapchain, None) }.unwrap();

        (swapchain, swapchain_images, surface_caps.current_extent)
    }
    fn create_device_and_queue(
        instance: &InstanceLoader,
        phys_device: &PhysicalDevice,
        queue_family_index: u32,
        device_extensions: &Vec<*const i8>,
        device_layers: &Vec<*const i8>,
    ) -> (DeviceLoader, Queue) {
        let queue_info = vec![vk::DeviceQueueCreateInfoBuilder::new()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0])];

        let phys_device_features = vk::PhysicalDeviceFeaturesBuilder::new();

        let device_info = vk::DeviceCreateInfoBuilder::new()
            .queue_create_infos(&queue_info)
            .enabled_features(&phys_device_features)
            .enabled_extension_names(device_extensions)
            .enabled_layer_names(device_layers);

        let device = DeviceLoader::new(instance, *phys_device, &device_info, None).unwrap();
        let queue = unsafe { device.get_device_queue(queue_family_index, 0, None) };

        (device, queue)
    }

    fn get_physical_device_data(
        instance: &InstanceLoader,
        phys_device: &PhysicalDevice,
        surface: &SurfaceKHR,
    ) -> Option<(SurfaceFormatKHR, PresentModeKHR, PhysicalDeviceProperties)> {
        unsafe {
            let mut format = None;
            let mut present_mode = None;
            let mut device_properties = None;

            //Quarrying format
            {
                let format_vec = instance
                    .get_physical_device_surface_formats_khr(*phys_device, *surface, None)
                    .unwrap();

                for surface_format in &format_vec {
                    if (surface_format.format == vk::Format::R8G8B8A8_SRGB)
                        && (surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR)
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
                let present_mode_vec = instance
                    .get_physical_device_surface_present_modes_khr(*phys_device, *surface, None)
                    .unwrap();

                for prsnt_mode in present_mode_vec {
                    if prsnt_mode == vk::PresentModeKHR::MAILBOX_KHR {
                        present_mode = Some(prsnt_mode);
                        break;
                    }
                }
                if present_mode == None {
                    present_mode = Some(vk::PresentModeKHR::FIFO_KHR)
                }
            }

            //Quarrying device properties
            {
                let device_extensions_needed = vec![vk::KHR_SWAPCHAIN_EXTENSION_NAME];
                let supported_device_extensions = instance
                    .enumerate_device_extension_properties(*phys_device, None, None)
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
                    Some(instance.get_physical_device_properties(*phys_device, None));
            }

            Some((
                format.unwrap(),
                present_mode.unwrap(),
                device_properties.unwrap(),
            ))
        }
    }

    fn get_physical_device(
        instance: &InstanceLoader,
        surface: &SurfaceKHR,
    ) -> Option<(PhysicalDevice, u32)> {
        unsafe {
            let phys_device_vec = instance.enumerate_physical_devices(None).unwrap();

            for phys_device in phys_device_vec {
                let queue_family_properties_vec =
                    instance.get_physical_device_queue_family_properties(phys_device, None);

                for (queue_family, queue_family_properties) in
                    queue_family_properties_vec.into_iter().enumerate()
                {
                    let queue_flags_valid = queue_family_properties
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS);
                    let device_surface_support = instance
                        .get_physical_device_surface_support_khr(
                            phys_device,
                            queue_family as u32,
                            *surface,
                            None,
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

    fn create_surface(instance: &InstanceLoader, window: &Window) -> SurfaceKHR {
        unsafe { surface::create_surface(instance, window, None).unwrap() }
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

    fn create_instance_loader(
        entry: &DefaultEntryLoader,
        window: &Window,
        debug: bool,
    ) -> InstanceLoader {
        let app_name = CString::new("ROT_Builder").unwrap();
        let eng_name = CString::new("ROT_Engine").unwrap();

        let app_info = vk::ApplicationInfoBuilder::new()
            .application_name(&app_name)
            .application_version(vk::make_version(0, 1, 0))
            .engine_version(vk::make_version(0, 1, 0))
            .engine_name(&eng_name)
            .api_version(make_version(1, 0, 0));

        let mut instance_extension = surface::enumerate_required_extensions(window).unwrap();

        let mut instance_layers = Vec::new();
        if debug {
            instance_extension.push(vk::EXT_DEBUG_UTILS_EXTENSION_NAME);
            instance_layers.push(cstr!("VK_LAYER_KHRONOS_validation"));
        }

        let instance_info = vk::InstanceCreateInfoBuilder::new()
            .application_info(&app_info)
            .enabled_layer_names(&instance_layers)
            .enabled_extension_names(&instance_extension);

        let instance_loader = InstanceLoader::new(&entry, &instance_info, None).unwrap();

        instance_loader
    }

    fn build_debug_calback(instance_loader: &InstanceLoader) -> DebugUtilsMessengerEXT {
        let messenger_info = vk::DebugUtilsMessengerCreateInfoEXTBuilder::new()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE_EXT
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING_EXT
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR_EXT
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO_EXT,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL_EXT
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION_EXT
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE_EXT,
            )
            .pfn_user_callback(Some(debug_callback));

        unsafe { instance_loader.create_debug_utils_messenger_ext(&messenger_info, None, None) }
            .unwrap()
    }
}

unsafe extern "system" fn debug_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagBitsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    match _message_severity {
        DebugUtilsMessageSeverityFlagBitsEXT(bit) => match bit {
            1 => {
                /*
                trace!(
                    "{}",
                    CStr::from_ptr((*p_callback_data).p_message).to_string_lossy()
                );
                */
            }
            16 => {
                info!(
                    "{}",
                    CStr::from_ptr((*p_callback_data).p_message).to_string_lossy()
                );
            }
            256 => {
                warn!(
                    "{}",
                    CStr::from_ptr((*p_callback_data).p_message).to_string_lossy()
                );
            }
            4096 => {
                error!(
                    "{}",
                    CStr::from_ptr((*p_callback_data).p_message).to_string_lossy()
                );
            }
            _ => {}
        },
    }

    vk::FALSE
}
