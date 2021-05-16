use ash::vk::{
    GraphicsPipelineCreateInfo, Pipeline, PipelineCache, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo, PipelineLayout,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineShaderStageCreateInfo, PipelineTessellationStateCreateInfo,
    PipelineViewportStateCreateInfo, RenderPass,
};

use ash::Device;

use ash::version::DeviceV1_0;

pub struct GraphicsPipelineState {
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
}

impl GraphicsPipelineState {
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
        }
    }

    pub fn enable_depth_testing(&mut self) {
        self.depth_stencil_state.depth_test_enable = 1;
    }

    pub fn enable_depth_writing(&mut self) {
        self.depth_stencil_state.depth_write_enable = 1;
    }

    pub fn set_vertex_shader_spirv(&mut self, spirv_code: &[u32]) {}

    pub fn build(&self, device: &Device) -> Pipeline {
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
            device
                .create_graphics_pipelines(PipelineCache::null(), &[create_info], None)
                .expect("Pipeline creation failed")[0]
        }
    }
}
