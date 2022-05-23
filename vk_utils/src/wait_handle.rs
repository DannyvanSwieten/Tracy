use std::rc::Rc;

use ash::vk::{CommandBuffer, CommandPool, Fence};

use crate::device_context::DeviceContext;

pub struct WaitHandle {
    device: Rc<DeviceContext>,
    command_pool: CommandPool,
    command_buffer: CommandBuffer,
    fence: Fence,
}

impl WaitHandle {
    pub(crate) fn new(
        device: Rc<DeviceContext>,
        command_pool: &CommandPool,
        command_buffer: CommandBuffer,
        fence: Fence,
    ) -> Self {
        Self {
            device: device.clone(),
            command_pool: command_pool.clone(),
            command_buffer,
            fence,
        }
    }

    pub fn has_completed(&self) -> bool {
        unsafe {
            match self.device.handle().wait_for_fences(&[self.fence], true, 0) {
                Err(_) => false,
                Ok(()) => true,
            }
        }
    }

    pub fn wait(&self) {
        unsafe {
            self.device
                .handle()
                .wait_for_fences(&[self.fence], true, std::u64::MAX)
                .expect("Wait failed");
        }
    }

    pub fn wait_for(&self, timeout: u64) -> bool {
        unsafe {
            match self
                .device
                .handle()
                .wait_for_fences(&[self.fence], true, timeout)
            {
                Err(_) => false,
                Ok(()) => true,
            }
        }
    }
}

impl Drop for WaitHandle {
    fn drop(&mut self) {
        unsafe {
            self.wait();
            self.device
                .handle()
                .free_command_buffers(self.command_pool, &[self.command_buffer])
        }
    }
}
