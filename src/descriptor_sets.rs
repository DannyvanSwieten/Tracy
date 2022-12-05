use std::rc::Rc;

use ash::vk::{
    DescriptorBufferInfo, DescriptorImageInfo, DescriptorPool, DescriptorPoolCreateInfo,
    DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayoutBinding,
    DescriptorSetLayoutCreateInfo, DescriptorType, ImageLayout, ImageView, PipelineLayout,
    PipelineLayoutCreateInfo, PushConstantRange, Sampler, ShaderStageFlags, WriteDescriptorSet,
    WriteDescriptorSetAccelerationStructureKHR,
};

use vk_utils::{buffer_resource::BufferResource, device_context::DeviceContext};

use crate::{ctx::GpuResources, geometry::TopLevelAccelerationStructure};
pub const ACCELERATION_STRUCTURE_LOCATION: (u32, u32) = (0, 0);
pub const OUTPUT_IMAGE_LOCATION: (u32, u32) = (0, 1);
pub const ACCUMULATION_IMAGE_LOCATION: (u32, u32) = (0, 2);

pub const CAMERA_BUFFER_LOCATION: (u32, u32) = (1, 0);
pub const BUFFER_ADDRESS_LOCATION: (u32, u32) = (1, 1);
pub const MESH_BUFFERS_LOCATION: (u32, u32) = (1, 2);
pub const MATERIAL_TEXTURE_LOCATION: (u32, u32) = (1, 3);
pub const SKYBOX_TEXTURE_LOCATION: (u32, u32) = (1, 4);

pub struct RTXDescriptorSets {
    pub max_sets: u32,
    pub next_set: u32,
    pub descriptor_pool: DescriptorPool,
    pub pipeline_layout: PipelineLayout,
    pub frame_descriptors: Vec<Rc<FrameDescriptors>>,
}

impl RTXDescriptorSets {
    pub fn new(device: Rc<DeviceContext>, max_sets: u32) -> Self {
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
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(BUFFER_ADDRESS_LOCATION.1),
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_BUFFER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(MESH_BUFFERS_LOCATION.1),
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1024)
                    .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(MATERIAL_TEXTURE_LOCATION.1),
                *DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .stage_flags(ShaderStageFlags::MISS_KHR)
                    .binding(SKYBOX_TEXTURE_LOCATION.1),
            ];

            let set_1 = *DescriptorSetLayoutCreateInfo::builder().bindings(&set_1_bindings);

            let descriptor_set_layouts = vec![
                device
                    .handle()
                    .create_descriptor_set_layout(&set_0, None)
                    .expect("Descriptor set layout creation failed"),
                device
                    .handle()
                    .create_descriptor_set_layout(&set_1, None)
                    .expect("Descriptor set layout creation failed"),
            ];

            let constant_ranges = [*PushConstantRange::builder()
                .size(8)
                .stage_flags(ShaderStageFlags::RAYGEN_KHR)];

            let pipeline_layout = device
                .handle()
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
                DescriptorPoolSize {
                    ty: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                },
            ];
            let descriptor_pool_create_info = DescriptorPoolCreateInfo::builder()
                .max_sets(max_sets * 2)
                .pool_sizes(&sizes);
            let descriptor_pool = device
                .handle()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Descriptor pool creation failed");

            let descriptor_set_create_info = DescriptorSetAllocateInfo::builder()
                .set_layouts(&descriptor_set_layouts)
                .descriptor_pool(descriptor_pool);

            let frame_descriptors = (0..max_sets)
                .into_iter()
                .map(|_| {
                    device
                        .handle()
                        .allocate_descriptor_sets(&descriptor_set_create_info)
                        .expect("Descriptor set allocation failed")
                })
                .map(|sets| {
                    Rc::new(FrameDescriptors {
                        device: device.clone(),
                        sets,
                    })
                })
                .collect();

            Self {
                max_sets,
                next_set: 0,
                pipeline_layout,
                descriptor_pool,
                frame_descriptors,
            }
        }
    }

    pub fn next(&mut self, resources: &GpuResources) -> Rc<FrameDescriptors> {
        let next = self.next_set;
        self.next_set += 1;
        self.next_set %= self.max_sets;
        Rc::get_mut(&mut self.frame_descriptors[next as usize])
            .unwrap()
            .update(resources);
        self.frame_descriptors[next as usize].clone()
    }
}

pub struct FrameDescriptors {
    pub device: Rc<DeviceContext>,
    pub sets: Vec<DescriptorSet>,
}

impl FrameDescriptors {
    pub fn new(device: Rc<DeviceContext>, sets: Vec<DescriptorSet>) -> Self {
        Self { sets, device }
    }

    pub fn update(&mut self, resources: &GpuResources) {
        self.update_acceleration_structure(&resources.acceleration_structure);
        self.update_images(&resources.image_views, &resources.sampler);
        self.update_buffer_address_buffer(&resources.buffer_address_buffer);
        self.update_geometry_address_buffer(&resources.geometry_address_buffer);
        self.update_camera_buffer(&resources.camera_buffer);
        self.update_output_images(&resources.output_image_views);
        self.update_skybox(&resources.skybox_image_view, &resources.sampler);
    }

    fn update_acceleration_structure(&self, acc_structure: &TopLevelAccelerationStructure) {
        let structures = [acc_structure.acceleration_structure];
        let mut acc_write = *WriteDescriptorSetAccelerationStructureKHR::builder()
            .acceleration_structures(&structures);

        let mut writes = [*WriteDescriptorSet::builder()
            .push_next(&mut acc_write)
            .dst_set(self.sets[ACCELERATION_STRUCTURE_LOCATION.0 as usize])
            .dst_binding(ACCELERATION_STRUCTURE_LOCATION.1)
            .descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR)];

        writes[0].descriptor_count = 1;

        unsafe {
            self.device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_images(&self, images: &[ImageView], sampler: &Sampler) {
        let image_infos: Vec<[DescriptorImageInfo; 1]> = images
            .iter()
            .map(|view| {
                [*DescriptorImageInfo::builder()
                    .image_view(*view)
                    .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .sampler(*sampler)]
            })
            .collect();

        let writes: Vec<WriteDescriptorSet> = image_infos
            .iter()
            .map(|info| {
                *WriteDescriptorSet::builder()
                    .image_info(info)
                    .dst_set(self.sets[MATERIAL_TEXTURE_LOCATION.0 as usize])
                    .dst_binding(MATERIAL_TEXTURE_LOCATION.1)
                    .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            })
            .collect();

        unsafe {
            self.device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_buffer_address_buffer(&self, buffer: &BufferResource) {
        let info = [*DescriptorBufferInfo::builder()
            .buffer(buffer.buffer)
            .range(buffer.content_size())];

        let writes = [*WriteDescriptorSet::builder()
            .buffer_info(&info)
            .dst_set(self.sets[BUFFER_ADDRESS_LOCATION.0 as usize])
            .dst_binding(BUFFER_ADDRESS_LOCATION.1)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)];

        unsafe {
            self.device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_geometry_address_buffer(&self, buffer: &BufferResource) {
        let info = [*DescriptorBufferInfo::builder()
            .buffer(buffer.buffer)
            .range(buffer.content_size())];

        let writes = [*WriteDescriptorSet::builder()
            .buffer_info(&info)
            .dst_set(self.sets[MESH_BUFFERS_LOCATION.0 as usize])
            .dst_binding(MESH_BUFFERS_LOCATION.1)
            .descriptor_type(DescriptorType::STORAGE_BUFFER)];

        unsafe {
            self.device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_camera_buffer(&self, buffer: &BufferResource) {
        let info = [*DescriptorBufferInfo::builder()
            .buffer(buffer.buffer)
            .range(buffer.content_size())];

        let writes = [*WriteDescriptorSet::builder()
            .buffer_info(&info)
            .dst_set(self.sets[CAMERA_BUFFER_LOCATION.0 as usize])
            .dst_binding(CAMERA_BUFFER_LOCATION.1)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)];

        unsafe {
            self.device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_output_images(&self, image_views: &[ImageView; 2]) {
        let image_writes = [*DescriptorImageInfo::builder()
            .image_view(image_views[0])
            .image_layout(ImageLayout::GENERAL)];
        let accumulation_image_writes = [*DescriptorImageInfo::builder()
            .image_view(image_views[1])
            .image_layout(ImageLayout::GENERAL)];

        let writes = [
            *WriteDescriptorSet::builder()
                .image_info(&image_writes)
                .dst_set(self.sets[OUTPUT_IMAGE_LOCATION.0 as usize])
                .dst_binding(OUTPUT_IMAGE_LOCATION.1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE),
            *WriteDescriptorSet::builder()
                .image_info(&accumulation_image_writes)
                .dst_set(self.sets[ACCUMULATION_IMAGE_LOCATION.0 as usize])
                .dst_binding(ACCUMULATION_IMAGE_LOCATION.1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE),
        ];

        unsafe {
            self.device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_skybox(&self, image_view: &ImageView, sampler: &Sampler) {
        let image_writes = [*DescriptorImageInfo::builder()
            .image_view(*image_view)
            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .sampler(*sampler)];

        let writes = [*WriteDescriptorSet::builder()
            .image_info(&image_writes)
            .dst_set(self.sets[SKYBOX_TEXTURE_LOCATION.0 as usize])
            .dst_binding(SKYBOX_TEXTURE_LOCATION.1)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)];

        unsafe {
            self.device.handle().update_descriptor_sets(&writes, &[]);
        }
    }
}
