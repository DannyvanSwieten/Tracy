use crate::memory::memory_type_index;

use ash::vk::{
    Buffer, BufferCreateInfo, BufferUsageFlags, DeviceMemory, MemoryAllocateInfo,
    MemoryPropertyFlags, PhysicalDeviceMemoryProperties, SharingMode,
};

use ash::version::DeviceV1_0;
use ash::Device;

pub struct BufferResource {
    device: Device,
    buffer: Buffer,
    memory: DeviceMemory,
}

impl BufferResource {
    pub fn new(
        properties: &PhysicalDeviceMemoryProperties,
        device: &Device,
        size: u64,
        property_flags: MemoryPropertyFlags,
        usage: BufferUsageFlags,
    ) -> Self {
        unsafe {
            let buffer_info = BufferCreateInfo::builder()
                .size(size)
                .sharing_mode(SharingMode::EXCLUSIVE)
                .usage(usage);

            let buffer = device
                .create_buffer(&buffer_info, None)
                .expect("Buffer creation failed");
            let memory_requirements = device.get_buffer_memory_requirements(buffer);
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
                    buffer,
                    memory,
                }
            } else {
                panic!()
            }
        }
    }
}

impl Drop for BufferResource {
    fn drop(&mut self) {
        unsafe { self.device.free_memory(self.memory, None) }
        unsafe { self.device.destroy_buffer(self.buffer, None) }
    }
}
