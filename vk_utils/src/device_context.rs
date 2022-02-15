use crate::buffer_resource::BufferResource;
use crate::gpu::Gpu;
use crate::image_resource::Image2DResource;
use crate::queue::QueueHandle;
use ash::vk::{
    BufferUsageFlags, DeviceCreateInfoBuilder, DeviceQueueCreateInfo, Format, ImageUsageFlags,
    MemoryPropertyFlags, QueueFlags,
};
use ash::Device;
use std::ffi::CStr;
pub struct DeviceContext {
    gpu: Gpu,
    device: Device,
    graphics_queue: Option<QueueHandle>,
}

impl DeviceContext {
    pub(crate) fn new(
        gpu: &Gpu,
        extensions: &[&'static CStr],
        builder: DeviceCreateInfoBuilder,
    ) -> Self {
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

            let builder = builder
                .enabled_extension_names(&extension_names_raw)
                .queue_create_infos(&queue_info);

            unsafe {
                let device_context: Device = gpu
                    .vulkan()
                    .vk_instance()
                    .create_device(*gpu.vk_physical_device(), &builder, None)
                    .unwrap();
                Self {
                    gpu: gpu.clone(),
                    device: device_context.clone(),
                    graphics_queue: Some(QueueHandle::new(
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

    pub fn wait(&self) {
        unsafe {
            self.device.device_wait_idle().expect("Wait failed");
        }
    }

    pub fn graphics_queue(&self) -> Option<&QueueHandle> {
        self.graphics_queue.as_ref()
    }
    pub fn vk_device(&self) -> &Device {
        &self.device
    }

    pub fn gpu(&self) -> &Gpu {
        &self.gpu
    }

    pub fn buffer(
        &self,
        size: u64,
        property_flags: MemoryPropertyFlags,
        usage: BufferUsageFlags,
    ) -> BufferResource {
        BufferResource::new(
            &self.gpu.memory_properties().memory_properties,
            self,
            size,
            property_flags,
            usage,
        )
    }

    pub fn image_2d(
        &self,
        width: u32,
        height: u32,
        format: Format,
        property_flags: MemoryPropertyFlags,
        usage: ImageUsageFlags,
    ) -> Image2DResource {
        Image2DResource::new(
            &self.gpu.memory_properties().memory_properties,
            self,
            width,
            height,
            format,
            usage,
            property_flags,
        )
    }
}
