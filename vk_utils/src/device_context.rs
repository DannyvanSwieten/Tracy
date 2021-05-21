use crate::gpu::Gpu;
use crate::graphics_queue::GraphicsQueue;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk::{DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice};
use ash::{Device, Instance};
pub struct DeviceContext {
    instance: Instance,
    physical_device: PhysicalDevice,
    graphics_queue: Option<GraphicsQueue>,
}

impl DeviceContext {
    pub(crate) fn new(instance: &Instance, gpu: &Gpu) {
        let priorities: [f32; 1] = [1.];
        // let queue_info = [DeviceQueueCreateInfo::builder()
        //     .queue_priorities(&priorities)
        //     .queue_family_index(physical_device_and_queue_family_index.1)
        //     .build()];
        // let device_create_info = DeviceCreateInfo::builder()
        //     .queue_create_infos(&queue_info)
        //     .enabled_extension_names(&device_extension_names_raw)
        //     .enabled_features(&features)
        //     .build();

        // let primary_device_context: Device = instance
        //     .create_device(*physical_device, &device_create_info, None)
        //     .unwrap();
        // Self {
        //     instance: instance.clone(),
        //     physical_device: physical_device_and_queue_family_index.0.clone(),
        //     graphics_queue: None,
        // }
    }

    pub fn graphics_queue(&self) {}
}
