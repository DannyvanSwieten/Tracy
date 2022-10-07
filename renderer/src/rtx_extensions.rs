use ash::extensions::khr::{AccelerationStructure, RayTracingPipeline};
use ash::vk::{PhysicalDeviceMemoryProperties2, PhysicalDeviceRayTracingPipelinePropertiesKHR};

use vk_utils::device_context::DeviceContext;
//

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
