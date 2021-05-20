use ash::vk::{Pipeline, RenderPass};
pub trait Canvas3D {
    fn begin(&mut self, renderpass: Option<&RenderPass>);
}