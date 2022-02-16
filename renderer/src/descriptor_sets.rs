use ash::vk::{
    DescriptorBufferInfo, DescriptorImageInfo, DescriptorPool, DescriptorPoolCreateInfo,
    DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout,
    DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, PipelineLayout,
    PipelineLayoutCreateInfo, ShaderStageFlags, WriteDescriptorSet,
};

use crate::scene_data::SceneData;
use vk_utils::device_context::DeviceContext;

pub struct RTXDescriptorSets {
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub descriptor_pool: DescriptorPool,
    pub pipeline_layout: PipelineLayout,
}

impl RTXDescriptorSets {
    pub fn new(device: &DeviceContext) -> Self {
        unsafe {
            let set_0_bindings = [
                // acceleration structure
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(0),
                // final image
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_IMAGE)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(1),
            ];

            let set_0 = *DescriptorSetLayoutCreateInfo::builder().bindings(&set_0_bindings);

            let set_1_bindings = [
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(0),
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR | ShaderStageFlags::RAYGEN_KHR)
                    .binding(1),
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(2)
                    .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(2),
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

            let pipeline_layout = device
                .vk_device()
                .create_pipeline_layout(
                    &PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_set_layouts),
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
                    ty: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 2,
                },
            ];
            let descriptor_pool_create_info = DescriptorPoolCreateInfo::builder()
                .max_sets(3)
                .pool_sizes(&sizes);
            let descriptor_pool = device
                .vk_device()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Descriptor pool creation failed");

            let descriptor_set_create_info = DescriptorSetAllocateInfo::builder()
                .set_layouts(&descriptor_set_layouts)
                .descriptor_pool(descriptor_pool);

            let descriptor_sets = device
                .vk_device()
                .allocate_descriptor_sets(&descriptor_set_create_info)
                .expect("Descriptor set allocation failed");

            Self {
                descriptor_set_layouts,
                pipeline_layout,
                descriptor_pool,
                descriptor_sets,
            }
        }
    }

    pub(crate) fn update_scene_descriptors(&self, ctx: &DeviceContext, scene_data: &SceneData) {
        let buffer_writes = [*DescriptorBufferInfo::builder()
            .range(scene_data.address_buffer.content_size())
            .buffer(scene_data.address_buffer.buffer)];

        let image_writes: Vec<DescriptorImageInfo> = scene_data
            .image_views
            .iter()
            .map(|view| {
                *DescriptorImageInfo::builder()
                    .image_view(*view)
                    .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .sampler(scene_data.samplers[0])
            })
            .collect();

        let writes = [*WriteDescriptorSet::builder()
            .dst_binding(1)
            .dst_set(self.descriptor_sets[1])
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&buffer_writes)];

        unsafe {
            ctx.vk_device().update_descriptor_sets(&writes, &[]);
        }

        let writes = [*WriteDescriptorSet::builder()
            .dst_binding(2)
            .dst_set(self.descriptor_sets[1])
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_writes)];
        unsafe {
            ctx.vk_device().update_descriptor_sets(&writes, &[]);
        }
    }
}
