#[allow(unused_imports)]
#[allow(non_camel_case_types)]
use log::{debug, error, info, trace, warn};

use crate::common::Vertex;
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::prelude::*;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, Device, Entry, Instance};
use std::borrow::Borrow;
use std::ffi::CString;
use std::fs;
use std::sync::Arc;
use winit::window::CursorIcon::Grab;

#[derive(Default)]
pub struct GraphicsPipeline {
    pub(crate) pipeline: vk::Pipeline,

    pub(crate) render_pass: vk::RenderPass,
    pub(crate) pipeline_layout: vk::PipelineLayout,

    vert_shader: vk::ShaderModule,
    frag_shader: vk::ShaderModule,
}

impl GraphicsPipeline {
    pub fn build(device: &Device, extent: &vk::Extent2D, format: &vk::SurfaceFormatKHR) -> Self {
        trace!("Digesting shaders and creating its modules");
        let (vert_shader_module, frag_shader_module) = GraphicsPipeline::load_shader(device);

        trace!("Creating shader stages");
        let entry_point = CString::new("main").unwrap();
        let shader_stages_vec = GraphicsPipeline::create_shader_stages(
            vert_shader_module,
            frag_shader_module,
            &entry_point,
        );

        trace!("Creating Vertex Input State");
        let bindind_desc = Vertex::get_binding_description();
        let attrib_desc = Vertex::get_attribute_description();
        let vertex = GraphicsPipeline::create_vertex_input_state(&bindind_desc, &attrib_desc);

        trace!("Creating Input Assembly State");
        let input_assembly = GraphicsPipeline::create_input_assembly_state();

        trace!("Creating Viewport State");
        let (viewports, scissors) = GraphicsPipeline::create_viewports_and_scissors(extent);
        let viewport = GraphicsPipeline::create_viewport_state(&viewports, &scissors);

        trace!("Building the Rasterizer State");
        let rasterizer = GraphicsPipeline::build_rasterizer_state();

        trace!("Creating Multisampling State");
        let multisampler = GraphicsPipeline::create_multisample_state();

        trace!("Creating Colorblender State");
        let color_blend_attachment = GraphicsPipeline::create_color_blend_attachment();
        let color_blender = GraphicsPipeline::create_color_blend_state(&color_blend_attachment);

        trace!("Creating Dynamic States");
        //let dynamic_states_wanted = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::LINE_WIDTH];
        let dynamic_states_wanted = Vec::new();
        let dynamic = GraphicsPipeline::create_dynamic_state(&dynamic_states_wanted);

        trace!("Creating Pipeline Layout");
        let pipeline_layout = GraphicsPipeline::create_pipeline_layout(device);

        trace!("Creating Renderpass");
        let render_pass = GraphicsPipeline::create_renderpass(device, format);

        trace!("Finally creating the Graphics Pipeline");
        let pipeline = GraphicsPipeline::build_graphics_pipeline(
            device,
            &shader_stages_vec,
            &vertex,
            &input_assembly,
            &viewport,
            &rasterizer,
            &multisampler,
            &color_blender,
            &dynamic,
            &pipeline_layout,
            &render_pass,
        );

        GraphicsPipeline {
            pipeline,
            render_pass,
            pipeline_layout,
            vert_shader: vert_shader_module,
            frag_shader: frag_shader_module,
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            device.destroy_pipeline(self.pipeline, None);

            device.destroy_render_pass(self.render_pass, None);

            device.destroy_pipeline_layout(self.pipeline_layout, None);

            device.destroy_shader_module(self.vert_shader, None);
            device.destroy_shader_module(self.frag_shader, None);
        }
    }
}

impl<'a> GraphicsPipeline {
    fn build_graphics_pipeline(
        device: &Device,
        shader_stages: &Vec<vk::PipelineShaderStageCreateInfo>,
        vertex_input: &vk::PipelineVertexInputStateCreateInfo,
        input_assembly: &vk::PipelineInputAssemblyStateCreateInfo,
        viewport: &vk::PipelineViewportStateCreateInfo,
        rasterization: &vk::PipelineRasterizationStateCreateInfo,
        multisample: &vk::PipelineMultisampleStateCreateInfo,
        color_blend: &vk::PipelineColorBlendStateCreateInfo,
        dynamic: &vk::PipelineDynamicStateCreateInfo,
        pipeline_layout: &vk::PipelineLayout,
        render_pass: &vk::RenderPass,
    ) -> vk::Pipeline {
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(input_assembly)
            .viewport_state(viewport)
            .rasterization_state(rasterization)
            .multisample_state(multisample)
            .color_blend_state(color_blend)
            .dynamic_state(dynamic)
            .layout(*pipeline_layout)
            .render_pass(*render_pass)
            .subpass(0)
            .build();

        let pipeline = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        }
        .unwrap()[0];

        pipeline
    }

    fn create_renderpass(device: &Device, format: &vk::SurfaceFormatKHR) -> vk::RenderPass {
        let attachments = vec![vk::AttachmentDescription::builder()
            .format(format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build()];

        let dependencies = vec![vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build()];

        let color_attachment_ref = vec![vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];

        let subpasses = vec![vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_ref)
            .build()];

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None) }.unwrap();

        render_pass
    }

    fn create_pipeline_layout(device: &Device) -> vk::PipelineLayout {
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder().build();

        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();

        pipeline_layout
    }

    fn create_dynamic_state(
        dynamic_states_wanted: &'a Vec<vk::DynamicState>,
    ) -> vk::PipelineDynamicStateCreateInfo {
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states_wanted)
            .build();

        dynamic_state
    }

    fn create_color_blend_state(
        colorblend_attachment_vec: &'a Vec<vk::PipelineColorBlendAttachmentState>,
    ) -> vk::PipelineColorBlendStateCreateInfo {
        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(colorblend_attachment_vec)
            .build();

        color_blending
    }

    fn create_color_blend_attachment() -> Vec<vk::PipelineColorBlendAttachmentState> {
        let mut colorblend_attachment_vec = Vec::new();

        let colorblender_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build();

        colorblend_attachment_vec.push(colorblender_attachment);

        colorblend_attachment_vec
    }

    fn create_multisample_state() -> vk::PipelineMultisampleStateCreateInfo {
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .build();

        multisample_state
    }

    fn build_rasterizer_state() -> vk::PipelineRasterizationStateCreateInfo {
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .build();

        rasterizer
    }

    fn create_viewport_state(
        viewports_vec: &'a Vec<vk::Viewport>,
        scissors_vec: &'a Vec<vk::Rect2D>,
    ) -> vk::PipelineViewportStateCreateInfo {
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(viewports_vec)
            .scissors(scissors_vec)
            .build();

        viewport_state
    }

    fn create_viewports_and_scissors(
        extent: &vk::Extent2D,
    ) -> (Vec<vk::Viewport>, Vec<vk::Rect2D>) {
        let mut viewports_vec = Vec::new();
        let mut scissors_vec = Vec::new();

        {
            let viewport_a = vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(extent.width as f32)
                .height(extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .build();

            viewports_vec.push(viewport_a);
        }

        {
            let scissor_a = vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(*extent)
                .build();

            scissors_vec.push(scissor_a);
        }

        (viewports_vec, scissors_vec)
    }

    fn create_input_assembly_state() -> vk::PipelineInputAssemblyStateCreateInfo {
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

        input_assembly
    }

    fn create_vertex_input_state(
        binding_descriptions: &'a [vk::VertexInputBindingDescription],
        attribute_descriptions: &'a [vk::VertexInputAttributeDescription],
    ) -> vk::PipelineVertexInputStateCreateInfo {
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(attribute_descriptions)
            .build();

        vertex_input
    }
    fn create_shader_stages(
        vertex_module: vk::ShaderModule,
        fragment_module: vk::ShaderModule,
        entry_point: &'a CString,
    ) -> Vec<vk::PipelineShaderStageCreateInfo> {
        let mut shader_stages = Vec::new();

        let vertex_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_module)
            .name(entry_point)
            .build();

        let fragment_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_module)
            .name(entry_point)
            .build();

        shader_stages.push(vertex_shader_stage);
        shader_stages.push(fragment_shader_stage);

        shader_stages
    }

    fn load_shader(device: &Device) -> (vk::ShaderModule, vk::ShaderModule) {
        //loading vertex shader
        let mut vert_bytes = fs::File::open("shaders/vert.spv").unwrap();
        let vert_decoded = ash::util::read_spv(&mut vert_bytes).unwrap();
        let module_info = vk::ShaderModuleCreateInfo::builder()
            .code(&vert_decoded)
            .build();
        let shader_vert = unsafe { device.create_shader_module(&module_info, None) }.unwrap();

        //loading fragment shader
        let mut frag_bytes = fs::File::open("shaders/frag.spv").unwrap();
        let frag_decoded = ash::util::read_spv(&mut frag_bytes).unwrap();
        let module_info = vk::ShaderModuleCreateInfo::builder()
            .code(&frag_decoded)
            .build();
        let shader_frag = unsafe { device.create_shader_module(&module_info, None) }.unwrap();

        (shader_vert, shader_frag)
    }
}
