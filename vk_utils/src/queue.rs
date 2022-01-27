use crate::command_buffer::CommandBufferHandle;
use crate::wait_handle::WaitHandle;
use ash::vk::{
    CommandBufferAllocateInfo, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo,
    Framebuffer, Queue, RenderPass,
};
use ash::Device;

#[derive(Clone)]
pub struct QueueHandle {
    device: Device,
    queue: Queue,
    queue_family_index: u32,
    command_pool: CommandPool,
}

impl QueueHandle {
    pub(crate) fn new(device: &Device, queue: &Queue, queue_family_index: u32) -> Self {
        let pool_info = CommandPoolCreateInfo::builder()
            .flags(CommandPoolCreateFlags::TRANSIENT)
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

    fn command_buffer(&self) -> CommandBufferHandle {
        let info = CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .command_buffer_count(1)
            .build();
        let command_buffer = unsafe {
            self.device
                .allocate_command_buffers(&info)
                .expect("Command Buffer allocation failed")[0]
        };
        CommandBufferHandle::new(&self.device, &self.queue, &command_buffer)
    }

    pub fn begin_render_pass<F>(
        &self,
        render_pass: &RenderPass,
        framebuffer: &Framebuffer,
        width: u32,
        height: u32,
        f: F,
    ) -> WaitHandle
    where
        F: FnOnce(CommandBufferHandle) -> CommandBufferHandle,
    {
        let command_buffer = self.command_buffer();
        command_buffer.begin();
        command_buffer.begin_render_pass(render_pass, framebuffer, width, height);
        let command_buffer = f(command_buffer);
        command_buffer.end_render_pass();
        command_buffer.submit(&self.command_pool)
    }

    pub fn begin<F>(&self, f: F) -> WaitHandle
    where
        F: FnOnce(CommandBufferHandle) -> CommandBufferHandle,
    {
        // unsafe {
        //     self.device
        //         .reset_command_pool(self.command_pool, CommandPoolResetFlags::RELEASE_RESOURCES)
        //         .expect("CommandPool reset failed");
        // }
        let command_buffer = self.command_buffer();
        command_buffer.begin();
        let command_buffer = f(command_buffer);
        command_buffer.submit(&self.command_pool)
    }
}
