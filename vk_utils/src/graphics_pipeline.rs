use ash::vk::{
    GraphicsPipelineCreateInfo, Pipeline, PipelineCache, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo, PipelineLayout,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineShaderStageCreateInfo, PipelineTessellationStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, RenderPass, ShaderModule, ShaderStageFlags,
};

use ash::Device;

use ash::version::DeviceV1_0;

pub struct GraphicsPipeline {
    blend_state: PipelineColorBlendStateCreateInfo,
    depth_stencil_state: PipelineDepthStencilStateCreateInfo,
    dynamic_state: PipelineDynamicStateCreateInfo,
    multisample_state: PipelineMultisampleStateCreateInfo,
    rasterization_state: PipelineRasterizationStateCreateInfo,
    shader_stage_state: Vec<PipelineShaderStageCreateInfo>,
    tesselation_state: PipelineTessellationStateCreateInfo,
    viewport_state: PipelineViewportStateCreateInfo,
    pipeline_layout: PipelineLayout,
    render_pass: RenderPass,
    pipeline: Pipeline,
}

impl GraphicsPipeline {
    pub fn new() -> Self {
        Self {
            blend_state: PipelineColorBlendStateCreateInfo::default(),
            depth_stencil_state: PipelineDepthStencilStateCreateInfo::default(),
            dynamic_state: PipelineDynamicStateCreateInfo::default(),
            multisample_state: PipelineMultisampleStateCreateInfo::default(),
            rasterization_state: PipelineRasterizationStateCreateInfo::default(),
            shader_stage_state: Vec::new(),
            tesselation_state: PipelineTessellationStateCreateInfo::default(),
            viewport_state: PipelineViewportStateCreateInfo::default(),
            pipeline_layout: PipelineLayout::null(),
            render_pass: RenderPass::null(),
            pipeline: Pipeline::null(),
        }
    }

    pub fn enable_depth_testing(&mut self) {
        self.depth_stencil_state.depth_test_enable = 1;
    }

    pub fn enable_depth_writing(&mut self) {
        self.depth_stencil_state.depth_write_enable = 1;
    }

    pub fn set_vertex_shader(&mut self, module: &ShaderModule) {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::VERTEX)
                .build(),
        )
    }

    pub fn set_fragment_shader(&mut self, module: &ShaderModule) {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::FRAGMENT)
                .build(),
        )
    }

    pub fn set_geometry_shader(&mut self, module: &ShaderModule) {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::GEOMETRY)
                .build(),
        )
    }

    pub fn set_tesselation_control_shader(&mut self, module: &ShaderModule) {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::TESSELLATION_CONTROL)
                .build(),
        )
    }

    pub fn set_tesselation_evaluation_shader(&mut self, module: &ShaderModule) {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::TESSELLATION_EVALUATION)
                .build(),
        )
    }

    pub fn build(&mut self, device: &Device) {
        let create_info = GraphicsPipelineCreateInfo::builder()
            .layout(self.pipeline_layout)
            .render_pass(self.render_pass)
            .stages(&self.shader_stage_state)
            .tessellation_state(&self.tesselation_state)
            .multisample_state(&self.multisample_state)
            .viewport_state(&self.viewport_state)
            .dynamic_state(&self.dynamic_state)
            .depth_stencil_state(&self.depth_stencil_state)
            .color_blend_state(&self.blend_state)
            .rasterization_state(&self.rasterization_state)
            .build();

        unsafe {
            self.pipeline = device
                .create_graphics_pipelines(PipelineCache::null(), &[create_info], None)
                .expect("Pipeline creation failed")[0];
        }
    }

    pub fn vk_handle(&self) -> &Pipeline {
        &self.pipeline
    }

    pub fn layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }
}
