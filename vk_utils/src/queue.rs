use std::rc::Rc;

use crate::device_context::DeviceContext;
use ash::vk::{CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Queue, QueueFlags};

#[derive(Clone)]
pub struct CommandQueue {
    device: Rc<DeviceContext>,
    handle: Queue,
    queue_family_index: u32,
    command_pool: CommandPool,
}

impl CommandQueue {
    pub fn new(device: Rc<DeviceContext>, flags: QueueFlags) -> Self {
        let queue_family_index = device.queue_family_index(flags).unwrap();

        let pool_info = CommandPoolCreateInfo::builder()
            .flags(CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue_family_index)
            .build();
        let command_pool = unsafe {
            device
                .handle()
                .create_command_pool(&pool_info, None)
                .expect("Command Pool Creation failed")
        };
        Self {
            device: device.clone(),
            handle: device.queue(queue_family_index),
            queue_family_index,
            command_pool,
        }
    }

    pub fn family_type_index(&self) -> u32 {
        self.queue_family_index
    }

    pub fn handle(&self) -> Queue {
        self.handle
    }

    pub(crate) fn pool(&self) -> CommandPool {
        self.command_pool
    }

    // pub fn begin_render_pass<F>(
    //     &self,
    //     render_pass: &RenderPass,
    //     framebuffer: &Framebuffer,
    //     width: u32,
    //     height: u32,
    //     f: F,
    // ) -> WaitHandle
    // where
    //     F: FnOnce(CommandBuffer) -> CommandBuffer,
    // {
    //     let command_buffer = self.command_buffer();
    //     command_buffer.begin();
    //     command_buffer.begin_render_pass(render_pass.handle(), framebuffer, width, height);
    //     let command_buffer = f(command_buffer);
    //     command_buffer.end_render_pass();
    //     command_buffer.submit(&self.command_pool)
    // }

    // pub fn begin<F>(&self, f: F) -> WaitHandle
    // where
    //     F: FnOnce(CommandBuffer) -> CommandBuffer,
    // {
    //     let command_buffer = self.command_buffer();
    //     command_buffer.begin();
    //     let command_buffer = f(command_buffer);
    //     command_buffer.submit(&self.command_pool)
    // }
}
