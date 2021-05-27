use crate::graphics_command_buffer::GraphicsCommandBuffer;
use crate::wait_handle::WaitHandle;
use ash::version::DeviceV1_0;
use ash::vk::{
    CommandBufferAllocateInfo, CommandPool, CommandPoolCreateInfo, Framebuffer, Queue, RenderPass,
};
use ash::Device;
pub struct GraphicsQueue {
    device: Device,
    queue: Queue,
    queue_family_index: u32,
    command_pool: CommandPool,
}

impl GraphicsQueue {
    pub(crate) fn new(device: &Device, queue: &Queue, queue_family_index: u32) -> Self {
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

    pub fn family_type_index(&self) -> u32 {
        self.queue_family_index
    }

    pub fn vk_queue(&self) -> &Queue {
        &self.queue
    }

    fn command_buffer(&self) -> GraphicsCommandBuffer {
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

    pub fn begin<F>(
        &self,
        render_pass: &RenderPass,
        framebuffer: &Framebuffer,
        width: u32,
        height: u32,
        f: F,
    ) -> WaitHandle
    where
        F: FnOnce(GraphicsCommandBuffer) -> GraphicsCommandBuffer,
    {
        let command_buffer = self.command_buffer();
        command_buffer.begin(render_pass, framebuffer, width, height);
        let command_buffer = f(command_buffer);
        command_buffer.submit(&self.command_pool)
    }
}
