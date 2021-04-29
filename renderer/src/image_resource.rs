use ash::vk::{
    DeviceMemory, Extent3D, Format, Image, ImageCreateInfo, ImageUsageFlags, MemoryAllocateInfo,
    MemoryRequirements, SampleCountFlags, SharingMode,
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
        device: &Device,
        width: u32,
        height: u32,
        format: Format,
        usage: ImageUsageFlags,
        memory_type_index: u32,
    ) -> Self {
        unsafe {
            let image_info = ImageCreateInfo::builder()
                .samples(SampleCountFlags::TYPE_1)
                .sharing_mode(SharingMode::EXCLUSIVE)
                .format(format)
                .extent(Extent3D::builder().width(width).height(height).build())
                .usage(usage);

            let image = device
                .create_image(&image_info, None)
                .expect("Image creation failed");
            let memory_requirements = device.get_image_memory_requirements(image);
            let allocation_info = MemoryAllocateInfo::builder()
                .memory_type_index(memory_type_index)
                .allocation_size(memory_requirements.size);
            let memory = device
                .allocate_memory(&allocation_info, None)
                .expect("Memory allocation failed");

            Self {
                device: device.clone(),
                image,
                memory,
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
