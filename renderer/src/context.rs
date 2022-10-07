use ash::extensions::khr::{AccelerationStructure, RayTracingPipeline};
use ash::vk::{
    CommandBuffer, CommandBufferAllocateInfo, CommandPool, CommandPoolCreateFlags,
    CommandPoolCreateInfo, Fence, PhysicalDeviceMemoryProperties2,
    PhysicalDeviceRayTracingPipelinePropertiesKHR, Queue, SubmitInfo,
};
use ash::Device;
use vk_utils::device_context::DeviceContext;
pub struct Context {
    device: Device,
    queue: Queue,
    command_pool: CommandPool,
}

impl Context {
    pub fn new(device: &Device, queue: &Queue, queue_family_index: u32) -> Self {
        let pool_info = CommandPoolCreateInfo::builder()
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        unsafe {
            let command_pool = device
                .create_command_pool(&pool_info, None)
                .expect("CommandPool creation Failed");

            Self {
                device: device.clone(),
                queue: queue.clone(),
                command_pool,
            }
        }
    }
    pub fn device(&self) -> &Device {
        &self.device
    }
    pub fn command_buffer(&self) -> CommandBuffer {
        let allocate_info = CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(self.command_pool);

        unsafe {
            self.device
                .allocate_command_buffers(&allocate_info)
                .expect("Commandbuffer allocation failed")[0]
        }
    }

    pub fn submit_command_buffers(&self, command_buffer: &CommandBuffer) {
        let submit_info = SubmitInfo::builder()
            .command_buffers(&[*command_buffer])
            .build();
        unsafe {
            self.device
                .queue_submit(self.queue, &[submit_info], Fence::null())
                .expect("Commandbuffer submit failed");
        }
    }
}

#[derive(Clone)]
pub struct RtxExtensions {
    acceleration_structure_ext: AccelerationStructure,
    ray_tracing_pipeline_ext: RayTracingPipeline,
    memory_properties: PhysicalDeviceMemoryProperties2,
    pipeline_properties: PhysicalDeviceRayTracingPipelinePropertiesKHR,
}

impl RtxExtensions {
    pub fn new(device: &DeviceContext) -> Self {
        let mut physical_device_memory_properties = PhysicalDeviceMemoryProperties2::default();
        unsafe {
            device
                .gpu()
                .vulkan()
                .vk_instance()
                .get_physical_device_memory_properties2(
                    *device.gpu().vk_physical_device(),
                    &mut physical_device_memory_properties,
                );
        }

        let acceleration_structure_ext =
            AccelerationStructure::new(device.gpu().vulkan().vk_instance(), device.handle());
        let ray_tracing_pipeline_ext =
            RayTracingPipeline::new(device.gpu().vulkan().vk_instance(), device.handle());

        let mut pipeline_properties = PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
        let _properties = device
            .gpu()
            .extension_properties(|builder| builder.push_next(&mut pipeline_properties));

        Self {
            acceleration_structure_ext,
            ray_tracing_pipeline_ext,
            memory_properties: physical_device_memory_properties,
            pipeline_properties: pipeline_properties,
        }
    }

    pub fn pipeline_ext(&self) -> &RayTracingPipeline {
        &self.ray_tracing_pipeline_ext
    }

    pub fn acceleration_structure_ext(&self) -> &AccelerationStructure {
        &self.acceleration_structure_ext
    }

    pub fn memory_properties(&self) -> &PhysicalDeviceMemoryProperties2 {
        &self.memory_properties
    }

    pub fn pipeline_properties(&self) -> &PhysicalDeviceRayTracingPipelinePropertiesKHR {
        &self.pipeline_properties
    }
}
