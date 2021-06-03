use ash::version::DeviceV1_0;
use ash::vk::{
    CommandBuffer, DependencyFlags, ImageAspectFlags, ImageLayout, ImageMemoryBarrier,
    ImageSubresourceRange, PipelineStageFlags, Queue, SubmitInfo,
};
use ash::Device;

use crate::image_resource::Image2DResource;

pub struct BaseCommandBuffer {
    device: Device,
    queue: Queue,
    command_buffer: [CommandBuffer; 1],
    submit_info: [SubmitInfo; 1],
}

impl BaseCommandBuffer {
    pub(crate) fn new(device: &Device, queue: &Queue, command_buffer: &CommandBuffer) -> Self {
        Self {
            device: device.clone(),
            queue: queue.clone(),
            command_buffer: [command_buffer.clone()],
            submit_info: [SubmitInfo::default()],
        }
    }

    pub(crate) fn device(&self) -> &Device {
        &self.device
    }

    pub(crate) fn queue(&self) -> &Queue {
        &self.queue
    }

    pub(crate) fn command_buffer(&self) -> &CommandBuffer {
        &self.command_buffer[0]
    }

    pub(crate) fn submit_info(&self) -> &[SubmitInfo] {
        &self.submit_info
    }

    pub fn image_transition(&self, image: &Image2DResource, layout: ImageLayout) {
        let barrier = ImageMemoryBarrier::builder()
            .old_layout(image.layout())
            .new_layout(layout)
            .image(*image.vk_image())
            .src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(
                ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1)
                    .build(),
            )
            .build();

        unsafe {
            self.device.cmd_pipeline_barrier(
                self.command_buffer[0],
                PipelineStageFlags::ALL_COMMANDS,
                PipelineStageFlags::ALL_COMMANDS,
                DependencyFlags::BY_REGION,
                &[],
                &[],
                &[barrier],
            );
        }
    }
}
