use ash::vk::RenderPass;
pub trait Canvas3D {
    fn begin(&mut self, renderpass: Option<&RenderPass>);
}
