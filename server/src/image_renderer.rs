use std::rc::Rc;

use vk_utils::{
    device_context::DeviceContext,
    graphics_pipeline::{GraphicsPipeline, GraphicsPipelineState},
};

pub struct ImageRenderer {
    device: Rc<DeviceContext>,
    pipeline: GraphicsPipeline,
}

impl ImageRenderer {
    pub fn new(device: Rc<DeviceContext>) {
        // let state = GraphicsPipelineState::new();
        //let pipeline = GraphicsPipeline::new(device, state)
    }
}
