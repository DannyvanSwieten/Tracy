use std::rc::Rc;

use ash::vk::{
    GraphicsPipelineCreateInfo, Pipeline, PipelineCache, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo, PipelineLayout,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineShaderStageCreateInfo, PipelineTessellationStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, RenderPass, ShaderModule, ShaderStageFlags,
};

use crate::device_context::DeviceContext;

#[derive(Default, Clone)]
pub struct GraphicsPipelineState {
    blend_state: Option<PipelineColorBlendStateCreateInfo>,
    depth_stencil_state: Option<PipelineDepthStencilStateCreateInfo>,
    dynamic_state: Option<PipelineDynamicStateCreateInfo>,
    multisample_state: Option<PipelineMultisampleStateCreateInfo>,
    rasterization_state: Option<PipelineRasterizationStateCreateInfo>,
    shader_stage_state: Vec<PipelineShaderStageCreateInfo>,
    tesselation_state: Option<PipelineTessellationStateCreateInfo>,
    viewport_state: Option<PipelineViewportStateCreateInfo>,
}

impl GraphicsPipelineState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_polygon_mode(mut self, mode: PolygonMode) -> Self {
        if self.rasterization_state.is_none() {
            self.rasterization_state = Some(PipelineRasterizationStateCreateInfo::default())
        }

        self.rasterization_state.unwrap().polygon_mode = mode;
        self
    }

    pub fn with_depth_testing(mut self) -> Self {
        if self.depth_stencil_state.is_none() {
            self.depth_stencil_state = Some(PipelineDepthStencilStateCreateInfo::default())
        }

        self.depth_stencil_state.unwrap().depth_test_enable = 1;
        self
    }

    pub fn with_depth_writing(mut self) -> Self {
        if self.depth_stencil_state.is_none() {
            self.depth_stencil_state = Some(PipelineDepthStencilStateCreateInfo::default())
        }
        self.depth_stencil_state.unwrap().depth_write_enable = 1;
        self
    }

    pub fn with_vertex_shader(mut self, module: &ShaderModule) -> Self {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::VERTEX)
                .build(),
        );

        self
    }

    pub fn with_fragment_shader(mut self, module: &ShaderModule) -> Self {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::FRAGMENT)
                .build(),
        );

        self
    }

    pub fn with_geometry_shader(mut self, module: &ShaderModule) -> Self {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::GEOMETRY)
                .build(),
        );

        self
    }

    pub fn with_tesselation_control_shader(mut self, module: &ShaderModule) -> Self {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::TESSELLATION_CONTROL)
                .build(),
        );

        self
    }

    pub fn with_tesselation_evaluation_shader(mut self, module: &ShaderModule) -> Self {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::TESSELLATION_EVALUATION)
                .build(),
        );

        self
    }
}

pub struct GraphicsPipeline {
    device: Rc<DeviceContext>,
    state: GraphicsPipelineState,
    pipeline_layout: PipelineLayout,
    pipeline: Pipeline,
}

impl GraphicsPipeline {
    pub fn new(device: Rc<DeviceContext>, state: GraphicsPipelineState) -> Self {
        Self {
            device,
            state,
            pipeline_layout: PipelineLayout::null(),
            pipeline: Pipeline::null(),
        }
    }

    pub fn handle(&self) -> &Pipeline {
        &self.pipeline
    }

    pub fn layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }
}
