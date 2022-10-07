use std::rc::Rc;

use crate::{context::RtxExtensions, gpu_scene::GpuTexture};
use vk_utils::{
    buffer_resource::BufferResource, command_buffer::CommandBuffer, device_context::DeviceContext,
    image2d_resource::Image2DResource, image_resource::ImageResource, queue::CommandQueue,
};

use crate::{gpu_resource::GpuResource, gpu_resource_cache::GpuResourceCache};

pub struct TextureImageData {
    pub format: ash::vk::Format,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl TextureImageData {
    pub fn new(format: ash::vk::Format, width: u32, height: u32, pixels: &[u8]) -> Self {
        if format == ash::vk::Format::R8G8B8_UNORM {
            let mut new_pixels = Vec::new();
            for i in (0..pixels.len()).step_by(3) {
                new_pixels.push(pixels[i]);
                new_pixels.push(pixels[i + 1]);
                new_pixels.push(pixels[i + 2]);
                new_pixels.push(255);
            }
            Self {
                format: ash::vk::Format::R8G8B8A8_UNORM,
                width,
                height,
                pixels: new_pixels,
            }
        } else {
            Self {
                format,
                width,
                height,
                pixels: pixels.to_vec(),
            }
        }
    }
}

impl GpuResource for TextureImageData {
    type Item = GpuTexture;

    fn prepare(
        &self,
        device: Rc<DeviceContext>,
        _: &RtxExtensions,
        queue: Rc<CommandQueue>,
        _: &GpuResourceCache,
    ) -> Self::Item {
        let mut image = Image2DResource::new(
            device.clone(),
            self.width,
            self.height,
            self.format,
            ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let mut buffer = BufferResource::new(
            device.clone(),
            self.pixels.len() as u64,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
            ash::vk::BufferUsageFlags::TRANSFER_SRC,
        );

        buffer.upload(&self.pixels);

        let mut command_buffer = CommandBuffer::new(device.clone(), queue.clone());
        command_buffer.begin();
        command_buffer
            .image_resource_transition(&mut image, ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL);

        command_buffer.copy_buffer_to_image(&buffer, &mut image);

        command_buffer
            .image_resource_transition(&mut image, ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        command_buffer.submit();

        let view_info = *ash::vk::ImageViewCreateInfo::builder()
            .format(self.format)
            .view_type(ash::vk::ImageViewType::TYPE_2D)
            .image(image.handle())
            .subresource_range(
                *ash::vk::ImageSubresourceRange::builder()
                    .layer_count(1)
                    .level_count(1)
                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR),
            );

        let image_view = unsafe {
            device
                .handle()
                .create_image_view(&view_info, None)
                .expect("Image view creation failed")
        };

        GpuTexture { image_view, image }
    }
}

pub struct TextureResource {
    pub id: usize,
    pub image: TextureImageData,
}
