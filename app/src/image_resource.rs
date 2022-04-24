use std::sync::Arc;

use renderer::{context::RtxContext, gpu_scene::GpuTexture};

use crate::{resource::GpuResource, resources::GpuResourceCache};

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
        device: &vk_utils::device_context::DeviceContext,
        _: &RtxContext,
        _: &GpuResourceCache,
    ) -> Self::Item {
        let mut image = device.image_2d(
            self.width,
            self.height,
            self.format,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
        );

        let mut buffer = device.buffer(
            self.pixels.len() as u64,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
            ash::vk::BufferUsageFlags::TRANSFER_SRC,
        );

        buffer.copy_to(&self.pixels);

        device
            .graphics_queue()
            .unwrap()
            .begin(|command_buffer_handle| {
                command_buffer_handle.color_image_resource_transition(
                    &mut image,
                    ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );

                command_buffer_handle
            });

        device
            .graphics_queue()
            .unwrap()
            .begin(|command_buffer_handle| {
                command_buffer_handle.copy_buffer_to_image_2d(&buffer, &image);
                command_buffer_handle
            });

        device
            .graphics_queue()
            .unwrap()
            .begin(|command_buffer_handle| {
                command_buffer_handle.color_image_resource_transition(
                    &mut image,
                    ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );

                command_buffer_handle
            });

        let view_info = *ash::vk::ImageViewCreateInfo::builder()
            .format(self.format)
            .view_type(ash::vk::ImageViewType::TYPE_2D)
            .image(*image.vk_image())
            .subresource_range(
                *ash::vk::ImageSubresourceRange::builder()
                    .layer_count(1)
                    .level_count(1)
                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR),
            );

        let image_view = unsafe {
            device
                .vk_device()
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
