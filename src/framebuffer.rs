use std::rc::Rc;

use ash::vk::{
    BufferUsageFlags, Format, ImageAspectFlags, ImageLayout, ImageSubresourceRange,
    ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, MemoryPropertyFlags,
};

use vk_utils::{
    buffer_resource::BufferResource, command_buffer::CommandBuffer, device_context::DeviceContext,
    image2d_resource::Image2DResource, image_resource::ImageResource, queue::CommandQueue,
};

use crate::math::Real;

pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    device: Rc<DeviceContext>,
    queue: Rc<CommandQueue>,
    pub accumulation_image: Image2DResource,
    pub accumulation_image_view: ImageView,
    pub final_image: Image2DResource,
    pub final_image_view: ImageView,
}

impl FrameBuffer {
    pub fn new(
        device: Rc<DeviceContext>,
        queue: Rc<CommandQueue>,
        width: u32,
        height: u32,
    ) -> Self {
        let final_image = Image2DResource::new(
            device.clone(),
            width as _,
            height as _,
            Format::R8G8B8A8_UNORM,
            ImageUsageFlags::STORAGE | ImageUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let accumulation_image = Image2DResource::new(
            device.clone(),
            width as _,
            height as _,
            Format::R32G32B32A32_SFLOAT,
            ImageUsageFlags::STORAGE,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let view_info = ImageViewCreateInfo::builder()
            .format(Format::R8G8B8A8_UNORM)
            .subresource_range(
                ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1)
                    .build(),
            )
            .view_type(ImageViewType::TYPE_2D)
            .image(final_image.handle());

        let final_image_view = unsafe {
            device
                .handle()
                .create_image_view(&view_info, None)
                .expect("Image View creation failed")
        };

        let accumulation_view_info = ImageViewCreateInfo::builder()
            .format(Format::R32G32B32A32_SFLOAT)
            .subresource_range(
                ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1)
                    .build(),
            )
            .view_type(ImageViewType::TYPE_2D)
            .image(accumulation_image.handle());

        let accumulation_image_view = unsafe {
            device
                .handle()
                .create_image_view(&accumulation_view_info, None)
                .expect("Image View creation failed")
        };

        Self {
            width,
            height,
            device,
            queue,
            accumulation_image,
            accumulation_image_view,
            final_image,
            final_image_view,
        }
    }

    pub fn aspect_ratio(&self) -> Real {
        self.width as Real / self.height as Real
    }

    pub fn download_output(&mut self) -> Vec<u8> {
        let size = self.width as u64 * self.height as u64 * 4;
        let buffer = BufferResource::new(
            self.device.clone(),
            size,
            MemoryPropertyFlags::DEVICE_LOCAL | MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::TRANSFER_DST,
        );
        self.device.wait();
        let mut command_buffer = CommandBuffer::new(self.device.clone(), self.queue.clone());
        command_buffer.begin();
        command_buffer
            .image_resource_transition(&mut self.final_image, ImageLayout::TRANSFER_SRC_OPTIMAL);
        command_buffer.copy_image_to_buffer(&self.final_image, &buffer);
        command_buffer.submit();
        self.device.wait();
        buffer.copy_data::<u8>()
    }
}
