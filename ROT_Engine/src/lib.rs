//! # ROT_GameEngine
//! this is where the magic happens
use rot_vk::{ROT_Renderer, Vertex2D};

mod log_rot;
use log::{debug, error, info, trace, warn};

use rot_layer;
use rot_layer::LayerStack;

use rot_events::{
    ROT_EventTranslator, ROT_Event_Base::ROT_Event, ROT_KeyboardInput::ROT_KeyboardInputEvent,
    ROT_MouseInput::ROT_MouseEvent,
};

use rot_gui::ROT_Gui;
use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::Thread;

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

use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};

/// This struct will define all the engine behavior.
/// Everything happens inside it.

pub struct ROT_Engine {
    //Buffer and Stacks
    layer_stack: LayerStack,

    //Renderer
    renderer: ROT_Renderer,

    //Event
    event_receiver: Option<Receiver<Arc<ROT_Event>>>,
    event_buffer: Vec<ROT_Event>,
}

// public impl block
impl ROT_Engine {
    pub fn build(dimensions: [u32; 2]) -> Self {
        //log inicialization, enables the debug, error, info, trace, warn macros
        log_rot::setup_logger().unwrap();
        //engine inicialization

        let mut layer_stack = LayerStack::new();
        let renderer = ROT_Renderer::build(dimensions);
        let gui = Box::new(ROT_Gui::build("GUI".to_string(), renderer.get_window()));
        layer_stack.push_indexed(gui, 0);

        let event_buffer = Vec::new();

        ROT_Engine {
            layer_stack,
            renderer,

            event_receiver: None,
            event_buffer,
        }
    }

    pub fn close() {}

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
            self.renderer.device.clone(),
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

        let vs = vs::Shader::load(self.renderer.device.clone()).unwrap();
        let fs = fs::Shader::load(self.renderer.device.clone()).unwrap();

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
            self.renderer.device.clone(),
            attachments:{
                color: {
                    load: Clear,
                    store: Store,
                    format: self.renderer.swapchain.format(),
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
                .build(self.renderer.device.clone())
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
        let mut framebuffers = ROT_Renderer::adjust_framebuffers(
            &self.renderer.images,
            render_pass.clone(),
            &mut dynamic_state,
        );
        let surface = self.renderer.surface.clone();
        let mut swapchain = self.renderer.swapchain.clone();
        let device = self.renderer.device.clone();
        let queue = self.renderer.queue.clone();

        let mut previous_frame_end = Some(sync::now(device.clone()).boxed());
        let mut recreate_swapchain = false;

        let mut event_loop = self.renderer.take_eventloop();

        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event: ev, .. } => match ev {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(_) => {
                        recreate_swapchain = true;
                    }
                    WindowEvent::KeyboardInput { .. } => {
                        let rot_event = ROT_EventTranslator::keyboard_input_event(ev).unwrap();
                        self.event_buffer.push(ROT_Event::KeyboardInput(rot_event));
                    }
                    WindowEvent::MouseWheel { .. }
                    | WindowEvent::MouseInput { .. }
                    | WindowEvent::CursorMoved { .. } => {
                        let rot_event = ROT_EventTranslator::mouse_event(ev).unwrap();
                        self.event_buffer.push(ROT_Event::MouseInput(rot_event));
                    }

                    _ => {}
                },
                Event::MainEventsCleared => {
                    self.dispatch_events();
                }
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

                        framebuffers = ROT_Renderer::adjust_framebuffers(
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
        })
    }

    fn dispatch_events(&mut self) {
        for event in &self.event_buffer {
            for layer in self.layer_stack.stack() {
                debug! {"{:?}", event}
                layer.on_event(event);
            }
        }
        self.event_buffer.clear();
    }

    fn update(&mut self, delta_time: f64) {
        for layer in self.layer_stack.stack() {
            layer.on_update(delta_time)
        }
    }
}
