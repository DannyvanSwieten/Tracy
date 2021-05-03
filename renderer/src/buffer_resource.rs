use crate::memory::memory_type_index;

use ash::vk::{
    Buffer, BufferCreateInfo, BufferUsageFlags, DeviceMemory, MemoryAllocateFlags,
    MemoryAllocateFlagsInfo, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags,
    PhysicalDeviceMemoryProperties, SharingMode,
};

use ash::version::{DeviceV1_0, DeviceV1_1, DeviceV1_2};
use ash::Device;

pub struct BufferResource {
    device: Device,
    pub buffer: Buffer,
    memory: DeviceMemory,
    size: u64,
}

impl BufferResource {
    pub fn copy_to<T>(&mut self, data: &[T]) {
        unsafe {
            let ptr = self
                .device
                .map_memory(self.memory, 0, self.size, MemoryMapFlags::default())
                .expect("Memory map failed on buffer");

            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                ptr as _,
                self.size as usize / std::mem::size_of::<T>(),
            );

            self.device.unmap_memory(self.memory);
        }
    }
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

            let mut flags = MemoryAllocateFlagsInfo::builder()
                .flags(MemoryAllocateFlags::DEVICE_ADDRESS_KHR)
                .build();
            if let Some(type_index) = type_index {
                let allocation_info = MemoryAllocateInfo::builder()
                    .push_next(&mut flags)
                    .memory_type_index(type_index)
                    .allocation_size(memory_requirements.size);
                let memory = device
                    .allocate_memory(&allocation_info, None)
                    .expect("Memory allocation failed");

                device
                    .bind_buffer_memory(buffer, memory, 0)
                    .expect("Buffer memory bind failed");

                Self {
                    device: device.clone(),
                    buffer,
                    memory,
                    size,
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
