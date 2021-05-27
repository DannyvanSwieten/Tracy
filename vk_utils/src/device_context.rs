use crate::gpu::Gpu;
use crate::graphics_queue::GraphicsQueue;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk::{DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, QueueFlags};
use ash::{Device, Instance};
use std::ffi::CStr;
pub struct DeviceContext {
    instance: Instance,
    physical_device: PhysicalDevice,
    device: Device,
    graphics_queue: Option<GraphicsQueue>,
}

impl DeviceContext {
    pub(crate) fn new(instance: &Instance, gpu: &Gpu, extensions: &[&'static CStr]) -> Self {
        let priorities: [f32; 1] = [1.];
        if let Some(index) = gpu.family_type_index(QueueFlags::GRAPHICS) {
            let queue_info = [DeviceQueueCreateInfo::builder()
                .queue_priorities(&priorities)
                .queue_family_index(index)
                .build()];

            let extension_names_raw: Vec<*const i8> = extensions
                .iter()
                .map(|layer_name| layer_name.as_ptr())
                .collect();

            let device_create_info = DeviceCreateInfo::builder()
                .enabled_extension_names(&extension_names_raw)
                .queue_create_infos(&queue_info)
                .build();
            unsafe {
                let device_context: Device = instance
                    .create_device(*gpu.vk_physical_device(), &device_create_info, None)
                    .unwrap();
                Self {
                    instance: instance.clone(),
                    physical_device: gpu.vk_physical_device().clone(),
                    device: device_context.clone(),
                    graphics_queue: Some(GraphicsQueue::new(
                        &device_context,
                        &device_context.get_device_queue(index, 0),
                        index,
                    )),
                }
            }
        } else {
            panic!()
        }
    }

    pub fn graphics_queue(&self) -> &Option<GraphicsQueue> {
        &self.graphics_queue
    }
    pub fn vk_device(&self) -> &Device {
        &self.device
    }
}
