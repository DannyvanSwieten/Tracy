use crate::device_context::DeviceContext;
use crate::memory::memory_type_index;

use ash::vk::{
    CommandBufferBeginInfo, DeviceMemory, Extent3D, Format, Image, ImageAspectFlags,
    ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, ImageType,
    ImageUsageFlags, MemoryAllocateInfo, MemoryPropertyFlags, PhysicalDeviceMemoryProperties,
    PipelineStageFlags, SampleCountFlags, SharingMode,
};

use ash::version::DeviceV1_0;
use ash::Device;

pub struct Image2DResource {
    device: Device,
    image: Image,
    memory: DeviceMemory,
    layout: ImageLayout,
}

impl Image2DResource {
    pub fn new(
        properties: &PhysicalDeviceMemoryProperties,
        context: &DeviceContext,
        width: u32,
        height: u32,
        format: Format,
        usage: ImageUsageFlags,
        property_flags: MemoryPropertyFlags,
    ) -> Self {
        unsafe {
            let image_info = ImageCreateInfo::builder()
                .image_type(ImageType::TYPE_2D)
                .samples(SampleCountFlags::TYPE_1)
                .sharing_mode(SharingMode::EXCLUSIVE)
                .format(format)
                .extent(
                    Extent3D::builder()
                        .width(width)
                        .height(height)
                        .depth(1)
                        .build(),
                )
                .array_layers(1)
                .mip_levels(1)
                .usage(usage);

            let device = context.vk_device();

            let image = device
                .create_image(&image_info, None)
                .expect("Image creation failed");
            let memory_requirements = device.get_image_memory_requirements(image);
            let type_index = memory_type_index(
                memory_requirements.memory_type_bits,
                properties,
                property_flags,
            );
            if let Some(type_index) = type_index {
                let allocation_info = MemoryAllocateInfo::builder()
                    .memory_type_index(type_index)
                    .allocation_size(memory_requirements.size);
                let memory = device
                    .allocate_memory(&allocation_info, None)
                    .expect("Memory allocation failed");

                device
                    .bind_image_memory(image, memory, 0)
                    .expect("Image memory bind failed");

                Self {
                    device: device.clone(),
                    image,
                    memory,
                    layout: ImageLayout::UNDEFINED,
                }
            } else {
                panic!()
            }
        }
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn layout(&self) -> ImageLayout {
        self.layout
    }

    pub fn transition(&self, ctx: &DeviceContext, new_layout: ImageLayout) {
        let barrier = ImageMemoryBarrier::builder()
            .old_layout(self.layout)
            .new_layout(new_layout)
            .image(self.image)
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

        let cmd = ctx.command_buffer();
        unsafe {
            ctx.device()
                .begin_command_buffer(cmd, &CommandBufferBeginInfo::builder().build())
                .expect("Begin commandbuffer failed");

            ctx.device().cmd_pipeline_barrier(
                cmd,
                PipelineStageFlags::ALL_COMMANDS,
                ash::vk::PipelineStageFlags::ALL_COMMANDS,
                ash::vk::DependencyFlags::BY_REGION,
                &[],
                &[],
                &[barrier],
            );

            ctx.device()
                .end_command_buffer(cmd)
                .expect("End command buffer failed");
        }
        ctx.submit_command_buffers(&cmd);
    }
}

impl Drop for Image2DResource {
    fn drop(&mut self) {
        unsafe { self.device.free_memory(self.memory, None) }
        unsafe { self.device.destroy_image(self.image, None) }
    }
}
