use ash::vk::{
    AttachmentDescription, AttachmentLoadOp, AttachmentReference, Device, Format, ImageLayout,
    RenderPassCreateInfo, SampleCountFlags, SubpassDependency, SubpassDescription,
};

pub struct RenderPass {
    device: Device,
    attachment_refs: Vec<AttachmentReference>,
    attachment_descriptions: Vec<AttachmentDescription>,
    subpass_dependencies: Vec<SubpassDependency>,
}
impl RenderPass {
    pub fn new(device: &Device) -> Self {
        Self {
            device: *device,
            attachment_refs: Vec::new(),
            attachment_descriptions: Vec::new(),
            subpass_dependencies: Vec::new(),
        }
    }

    pub fn with_color_attachment(mut self, format: Format, final_layout: ImageLayout) -> Self {
        let attachment_description = AttachmentDescription::builder()
            .final_layout(final_layout)
            .load_op(AttachmentLoadOp::LOAD)
            .samples(SampleCountFlags::TYPE_1)
            .format(format)
            .build();

        let attachment_ref = AttachmentReference::builder()
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .attachment(self.attachment_descriptions.len() as u32)
            .build();

        self.attachment_descriptions.push(attachment_description);
        self.attachment_refs.push(attachment_ref);

        self
    }

    pub fn with_depth_attachment(mut self, format: Format) -> Self {
        let attachment_description = AttachmentDescription::builder()
            .load_op(AttachmentLoadOp::DONT_CARE)
            .samples(SampleCountFlags::TYPE_1)
            .format(format)
            .build();

        let attachment_ref = AttachmentReference::builder()
            .layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .attachment(self.attachment_descriptions.len() as u32)
            .build();

        self.attachment_descriptions.push(attachment_description);
        self.attachment_refs.push(attachment_ref);

        self
    }

    pub fn build(&mut self, device: &Device) {
        let info = RenderPassCreateInfo::builder()
            .attachments(&self.attachment_descriptions)
            .build();
    }
}
