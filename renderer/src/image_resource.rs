use crate::memory::memory_type_index;

use ash::vk::{
    DeviceMemory, Extent3D, Format, Image, ImageCreateInfo, ImageType, ImageUsageFlags,
    MemoryAllocateInfo, MemoryPropertyFlags, PhysicalDeviceMemoryProperties, SampleCountFlags,
    SharingMode,
};

use ash::version::DeviceV1_0;
use ash::Device;

pub struct Image2DResource {
    device: Device,
    image: Image,
    memory: DeviceMemory,
}

impl Image2DResource {
    pub fn new(
        properties: &PhysicalDeviceMemoryProperties,
        device: &Device,
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

                Self {
                    device: device.clone(),
                    image,
                    memory,
                }
            } else {
                panic!()
            }
        }
    }
}

impl Drop for Image2DResource {
    fn drop(&mut self) {
        unsafe { self.device.free_memory(self.memory, None) }
        unsafe { self.device.destroy_image(self.image, None) }
    }
}
