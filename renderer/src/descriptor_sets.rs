use ash::vk::{
    DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet,
    DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding,
    DescriptorSetLayoutCreateInfo, DescriptorType, PipelineLayout, PipelineLayoutCreateInfo,
    PushConstantRange, ShaderStageFlags,
};

use vk_utils::device_context::DeviceContext;

pub const CAMERA_BUFFER_LOCATION: (u32, u32) = (1, 0);
pub const MATERIAL_BUFFER_ADDRESS_LOCATION: (u32, u32) = (1, 1);
pub const MATERIAL_TEXTURE_LOCATION: (u32, u32) = (1, 2);
pub const MESH_BUFFERS_LOCATION: (u32, u32) = (1, 3);

pub const ACCELERATION_STRUCTURE_LOCATION: (u32, u32) = (0, 0);
pub const OUTPUT_IMAGE_LOCATION: (u32, u32) = (0, 1);
pub const ACCUMULATION_IMAGE_LOCATION: (u32, u32) = (0, 2);

pub struct RTXDescriptorSets {
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
    pub descriptor_pool: DescriptorPool,
    pub pipeline_layout: PipelineLayout,
}

impl RTXDescriptorSets {
    pub fn descriptor_sets(&self, device: &DeviceContext) -> Vec<DescriptorSet> {
        let descriptor_set_create_info = DescriptorSetAllocateInfo::builder()
            .set_layouts(&self.descriptor_set_layouts)
            .descriptor_pool(self.descriptor_pool);
        unsafe {
            device
                .vk_device()
                .allocate_descriptor_sets(&descriptor_set_create_info)
                .expect("Descriptor set allocation failed")
        }
    }

    pub fn new(device: &DeviceContext) -> Self {
        unsafe {
            let set_0_bindings = [
                // acceleration structure
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR | ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(ACCELERATION_STRUCTURE_LOCATION.1),
                // final image
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_IMAGE)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(OUTPUT_IMAGE_LOCATION.1),
                // accumulation image
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_IMAGE)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(ACCUMULATION_IMAGE_LOCATION.1),
            ];

            let set_0 = *DescriptorSetLayoutCreateInfo::builder().bindings(&set_0_bindings);

            let set_1_bindings = [
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(CAMERA_BUFFER_LOCATION.1),
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR | ShaderStageFlags::RAYGEN_KHR)
                    .binding(MATERIAL_BUFFER_ADDRESS_LOCATION.1),
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1024)
                    .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(MATERIAL_TEXTURE_LOCATION.1),
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_BUFFER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(MESH_BUFFERS_LOCATION.1),
            ];

            let set_1 = *DescriptorSetLayoutCreateInfo::builder().bindings(&set_1_bindings);

            let descriptor_set_layouts = vec![
                device
                    .vk_device()
                    .create_descriptor_set_layout(&set_0, None)
                    .expect("Descriptor set layout creation failed"),
                device
                    .vk_device()
                    .create_descriptor_set_layout(&set_1, None)
                    .expect("Descriptor set layout creation failed"),
            ];

            let constant_ranges = [*PushConstantRange::builder()
                .size(8)
                .stage_flags(ShaderStageFlags::RAYGEN_KHR)];

            let pipeline_layout = device
                .vk_device()
                .create_pipeline_layout(
                    &PipelineLayoutCreateInfo::builder()
                        .set_layouts(&descriptor_set_layouts)
                        .push_constant_ranges(&constant_ranges),
                    None,
                )
                .expect("Pipeline layout creation failed");

            let sizes = [
                // scene
                DescriptorPoolSize {
                    ty: DescriptorType::ACCELERATION_STRUCTURE_KHR,
                    descriptor_count: 1,
                },
                // accumulation + output image
                DescriptorPoolSize {
                    ty: DescriptorType::STORAGE_IMAGE,
                    descriptor_count: 2,
                },
                // camera
                DescriptorPoolSize {
                    ty: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 2,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 1,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1024,
                },
            ];
            let descriptor_pool_create_info = DescriptorPoolCreateInfo::builder()
                .max_sets(32)
                .pool_sizes(&sizes);
            let descriptor_pool = device
                .vk_device()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Descriptor pool creation failed");

            Self {
                descriptor_set_layouts,
                pipeline_layout,
                descriptor_pool,
            }
        }
    }
}
