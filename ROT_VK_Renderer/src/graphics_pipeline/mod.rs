#[allow(unused_imports)]
#[allow(non_camel_case_types)]
use log::{debug, error, info, trace, warn};

use erupt::vk::{
    DynamicState, Extent2D, Pipeline, PipelineColorBlendAttachmentStateBuilder,
    PipelineColorBlendStateCreateInfoBuilder, PipelineInputAssemblyStateCreateInfoBuilder,
    PipelineLayout, PipelineRasterizationStateCreateInfoBuilder,
    PipelineShaderStageCreateInfoBuilder, PipelineVertexInputStateCreateInfoBuilder,
    PipelineViewportStateCreateInfoBuilder, RenderPass, ShaderModule, SurfaceFormatKHR,
    ViewportBuilder,
};
use erupt::vk1_0::{
    PipelineDynamicStateCreateInfoBuilder, PipelineMultisampleStateCreateInfoBuilder, Rect2DBuilder,
};
use erupt::{utils, vk, DeviceLoader};
use std::borrow::Borrow;
use std::ffi::CString;
use std::fs;
use std::sync::Arc;
use winit::window::CursorIcon::Grab;

#[derive(Default)]
pub struct GraphicsPipeline {
    pub(crate) pipeline: Pipeline,

    pub(crate) render_pass: RenderPass,
    pub(crate) pipeline_layout: PipelineLayout,

    vert_shader: ShaderModule,
    frag_shader: ShaderModule,
}

impl GraphicsPipeline {
    pub fn build(device: &DeviceLoader, extent: &Extent2D, format: &SurfaceFormatKHR) -> Self {
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
        let vertex = GraphicsPipeline::create_vertex_input_state();

        trace!("Creating Input Assembly State");
        let input_assembly = GraphicsPipeline::create_input_assembly_state();

        trace!("Creating Viewport State");
        let (viewports, scissors) = GraphicsPipeline::create_viewports_and_scissors_builder(extent);
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

    pub fn destroy(&mut self, device: &DeviceLoader) {
        unsafe {
            device.destroy_pipeline(Some(self.pipeline), None);

            device.destroy_render_pass(Some(self.render_pass), None);

            device.destroy_pipeline_layout(Some(self.pipeline_layout), None);

            device.destroy_shader_module(Some(self.vert_shader), None);
            device.destroy_shader_module(Some(self.frag_shader), None);
        }
    }
}

impl<'a> GraphicsPipeline {
    fn build_graphics_pipeline(
        device: &DeviceLoader,
        shader_stages: &Vec<PipelineShaderStageCreateInfoBuilder>,
        vertex_input: &PipelineVertexInputStateCreateInfoBuilder,
        input_assembly: &PipelineInputAssemblyStateCreateInfoBuilder,
        viewport: &PipelineViewportStateCreateInfoBuilder,
        rasterization: &PipelineRasterizationStateCreateInfoBuilder,
        multisample: &PipelineMultisampleStateCreateInfoBuilder,
        color_blend: &PipelineColorBlendStateCreateInfoBuilder,
        dynamic: &PipelineDynamicStateCreateInfoBuilder,
        pipeline_layout: &PipelineLayout,
        render_pass: &RenderPass,
    ) -> Pipeline {
        let pipeline_info = vk::GraphicsPipelineCreateInfoBuilder::new()
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
            .subpass(0);

        let pipeline =
            unsafe { device.create_graphics_pipelines(None, &[pipeline_info], None) }.unwrap()[0];

        pipeline
    }

    fn create_renderpass(device: &DeviceLoader, format: &SurfaceFormatKHR) -> RenderPass {
        let attachments = vec![vk::AttachmentDescriptionBuilder::new()
            .format(format.format)
            .samples(vk::SampleCountFlagBits::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

        let dependencies = vec![vk::SubpassDependencyBuilder::new()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)];

        let color_attachment_ref = vec![vk::AttachmentReferenceBuilder::new()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        let subpasses = vec![vk::SubpassDescriptionBuilder::new()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_ref)];

        let render_pass_info = vk::RenderPassCreateInfoBuilder::new()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let render_pass =
            unsafe { device.create_render_pass(&render_pass_info, None, None) }.unwrap();

        render_pass
    }

    fn create_pipeline_layout(device: &DeviceLoader) -> PipelineLayout {
        let pipeline_layout_info = vk::PipelineLayoutCreateInfoBuilder::new();

        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None, None) }.unwrap();

        pipeline_layout
    }

    fn create_dynamic_state(
        dynamic_states_wanted: &'a Vec<DynamicState>,
    ) -> PipelineDynamicStateCreateInfoBuilder<'a> {
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfoBuilder::new().dynamic_states(&dynamic_states_wanted);

        dynamic_state
    }

    fn create_color_blend_state(
        colorblend_attachment_vec: &'a Vec<PipelineColorBlendAttachmentStateBuilder>,
    ) -> PipelineColorBlendStateCreateInfoBuilder<'a> {
        let color_blending = vk::PipelineColorBlendStateCreateInfoBuilder::new()
            .logic_op_enable(false)
            .attachments(colorblend_attachment_vec);

        color_blending
    }

    fn create_color_blend_attachment() -> Vec<PipelineColorBlendAttachmentStateBuilder<'a>> {
        let mut colorblend_attachment_vec = Vec::new();

        let colorblender_attachment = vk::PipelineColorBlendAttachmentStateBuilder::new()
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
            .alpha_blend_op(vk::BlendOp::ADD);

        colorblend_attachment_vec.push(colorblender_attachment);

        colorblend_attachment_vec
    }

    fn create_multisample_state() -> PipelineMultisampleStateCreateInfoBuilder<'a> {
        let multisample_state = vk::PipelineMultisampleStateCreateInfoBuilder::new()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlagBits::_1);

        multisample_state
    }

    fn build_rasterizer_state() -> PipelineRasterizationStateCreateInfoBuilder<'a> {
        let rasterizer = vk::PipelineRasterizationStateCreateInfoBuilder::new()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        rasterizer
    }

    fn create_viewport_state(
        viewports_vec: &'a Vec<ViewportBuilder>,
        scissors_vec: &'a Vec<Rect2DBuilder>,
    ) -> PipelineViewportStateCreateInfoBuilder<'a> {
        let viewport_state = vk::PipelineViewportStateCreateInfoBuilder::new()
            .viewports(viewports_vec)
            .scissors(scissors_vec);

        viewport_state
    }

    fn create_viewports_and_scissors_builder(
        extent: &Extent2D,
    ) -> (Vec<ViewportBuilder>, Vec<Rect2DBuilder<'a>>) {
        let mut viewports_vec = Vec::new();
        let mut scissors_vec = Vec::new();

        {
            let viewport_a = vk::ViewportBuilder::new()
                .x(0.0)
                .y(0.0)
                .width(extent.width as f32)
                .height(extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0);

            viewports_vec.push(viewport_a);
        }

        {
            let scissor_a = vk::Rect2DBuilder::new()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(*extent);

            scissors_vec.push(scissor_a);
        }

        (viewports_vec, scissors_vec)
    }

    fn create_input_assembly_state() -> PipelineInputAssemblyStateCreateInfoBuilder<'a> {
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        input_assembly
    }

    fn create_vertex_input_state() -> PipelineVertexInputStateCreateInfoBuilder<'a> {
        let vertex_input = vk::PipelineVertexInputStateCreateInfoBuilder::new();

        vertex_input
    }
    fn create_shader_stages(
        vertex_module: ShaderModule,
        fragment_module: ShaderModule,
        entry_point: &'a CString,
    ) -> Vec<PipelineShaderStageCreateInfoBuilder<'a>> {
        let mut shader_stages = Vec::new();

        let vertex_shader_stage = vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(vk::ShaderStageFlagBits::VERTEX)
            .module(vertex_module)
            .name(entry_point);

        let fragment_shader_stage = vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(vk::ShaderStageFlagBits::FRAGMENT)
            .module(fragment_module)
            .name(entry_point);

        shader_stages.push(vertex_shader_stage);
        shader_stages.push(fragment_shader_stage);

        shader_stages
    }

    fn load_shader(device: &DeviceLoader) -> (ShaderModule, ShaderModule) {
        //loading vertex shader
        let vert_bytes = fs::read("shaders\\vert.spv").unwrap();
        let vert_decoded = utils::decode_spv(&vert_bytes).unwrap();
        let module_info = vk::ShaderModuleCreateInfoBuilder::new().code(&vert_decoded);
        let shader_vert = unsafe { device.create_shader_module(&module_info, None, None) }.unwrap();

        //loading fragment shader
        let frag_bytes = fs::read("shaders\\frag.spv").unwrap();
        let frag_decoded = utils::decode_spv(&frag_bytes).unwrap();
        let module_info = vk::ShaderModuleCreateInfoBuilder::new().code(&frag_decoded);
        let shader_frag = unsafe { device.create_shader_module(&module_info, None, None) }.unwrap();

        (shader_vert, shader_frag)
    }
}
