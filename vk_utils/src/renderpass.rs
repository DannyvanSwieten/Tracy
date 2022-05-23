use std::{ptr::null, rc::Rc};

use ash::vk::{
    AttachmentDescription, AttachmentLoadOp, AttachmentReference, Device, Format, ImageLayout,
    RenderPassCreateInfo, SampleCountFlags, SubpassDependency, SubpassDescription,
};

use crate::{device_context::DeviceContext, swapchain::Swapchain};

pub struct RenderPass {
    device: Rc<DeviceContext>,
    attachment_refs: Vec<AttachmentReference>,
    attachment_descriptions: Vec<AttachmentDescription>,
    subpass_dependencies: Vec<SubpassDependency>,
    subpass_descriptions: Vec<SubpassDescription>,
    handle: ash::vk::RenderPass,
}
impl RenderPass {
    pub fn from_swapchain(device: Rc<DeviceContext>, swapchain: &Swapchain) -> Self {
        let attachment_descriptions = vec![ash::vk::AttachmentDescription {
            format: *swapchain.format(),
            samples: ash::vk::SampleCountFlags::TYPE_1,
            load_op: ash::vk::AttachmentLoadOp::DONT_CARE,
            store_op: ash::vk::AttachmentStoreOp::STORE,
            final_layout: ash::vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        }];

        let attachment_refs = vec![ash::vk::AttachmentReference {
            attachment: 0,
            layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let subpass_dependencies = vec![ash::vk::SubpassDependency {
            src_subpass: ash::vk::SUBPASS_EXTERNAL,
            src_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_READ
                | ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpass_descriptions = vec![ash::vk::SubpassDescription::builder()
            .color_attachments(&attachment_refs)
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
            .build()];

        let renderpass_create_info = ash::vk::RenderPassCreateInfo::builder()
            .attachments(&attachment_descriptions)
            .subpasses(&subpass_descriptions)
            .dependencies(&subpass_dependencies);

        let handle = unsafe {
            device
                .handle()
                .create_render_pass(&renderpass_create_info, None)
                .expect("Renderpass creation failed for swapchain")
        };

        Self {
            device: device.clone(),
            attachment_descriptions,
            attachment_refs,
            subpass_dependencies,
            subpass_descriptions,
            handle,
        }
    }

    pub fn new(device: Rc<DeviceContext>) -> Self {
        Self {
            device: device.clone(),
            attachment_refs: Vec::new(),
            attachment_descriptions: Vec::new(),
            subpass_dependencies: Vec::new(),
            subpass_descriptions: Vec::new(),
            handle: ash::vk::RenderPass::null(),
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

    pub fn with_sub_pass_dependency(mut self, dependency: SubpassDependency) -> Self {
        self.subpass_dependencies.push(dependency);
        self
    }

    pub fn build(&mut self) {
        let info = RenderPassCreateInfo::builder()
            .attachments(&self.attachment_descriptions)
            .build();
    }

    pub fn handle(&self) -> &ash::vk::RenderPass {
        &self.handle
    }
}
