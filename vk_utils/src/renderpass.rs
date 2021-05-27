use ash::vk::{
    AttachmentDescription, AttachmentLoadOp, AttachmentReference, Device, Format, ImageLayout,
    RenderPassCreateInfo, SampleCountFlags,
};

pub struct RenderPass {
    attachment_refs: Vec<AttachmentReference>,
    attachment_descriptions: Vec<AttachmentDescription>,
}
impl RenderPass {
    pub fn add_color_attachment(&mut self, format: Format) {
        let attachment_description = AttachmentDescription::builder()
            .load_op(AttachmentLoadOp::DONT_CARE)
            .samples(SampleCountFlags::TYPE_1)
            .format(format)
            .build();

        let attachment_ref = AttachmentReference::builder()
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .attachment(self.attachment_descriptions.len() as u32)
            .build();

        self.attachment_descriptions.push(attachment_description);
        self.attachment_refs.push(attachment_ref);
    }

    pub fn build(&mut self, device: &Device) {
        let info = RenderPassCreateInfo::builder()
            .attachments(&self.attachment_descriptions)
            .build();
    }
}
