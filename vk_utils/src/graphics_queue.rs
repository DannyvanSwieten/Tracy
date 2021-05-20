use crate::graphics_command_buffer::GraphicsCommandBuffer;

use ash::version::DeviceV1_0;
use ash::vk::{CommandBufferAllocateInfo, CommandPool, CommandPoolCreateInfo, Queue};
use ash::Device;
pub struct GraphicsQueue {
    device: Device,
    queue: Queue,
    command_pool: CommandPool,
    queue_family_index: u32,
}

impl GraphicsQueue {
    fn new(device: &Device, queue: &Queue, queue_family_index: u32) -> Self {
        let pool_info = CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .build();
        let command_pool = unsafe {
            device
                .create_command_pool(&pool_info, None)
                .expect("Command Pool Creation failed")
        };
        Self {
            device: device.clone(),
            queue: queue.clone(),
            queue_family_index,
            command_pool,
        }
    }

    pub fn command_buffer(&self) -> GraphicsCommandBuffer {
        let info = CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .command_buffer_count(1)
            .build();
        let command_buffer = unsafe {
            self.device
                .allocate_command_buffers(&info)
                .expect("Command Buffer allocation failed")[0]
        };
        GraphicsCommandBuffer::new(&self.device, &self.queue, &command_buffer)
    }
}
