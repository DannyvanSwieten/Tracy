use ash::vk::{Pipeline, RenderPass};
pub trait Canvas3D {
    fn begin_render_pass(&mut self, renderpass: &RenderPass);
    fn begin_pipeline(&mut self, pipeline: &Pipeline);
    fn end_pipeline(&mut self);
    fn swapchain_render_pass(&self) -> &RenderPass;
}