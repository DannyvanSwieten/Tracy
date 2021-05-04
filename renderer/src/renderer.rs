use crate::buffer_resource::BufferResource;
use crate::context::RtxContext;
use crate::geometry::{
    BottomLevelAccelerationStructure, GeometryInstance, TopLevelAccelerationStructure,
};
use crate::image_resource::Image2DResource;
use crate::spirv::load_spirv;

// Extension functions
use ash::extensions::khr::{AccelerationStructure, DeferredHostOperations, RayTracingPipeline};

// Version traits
use ash::version::{DeviceV1_0, InstanceV1_0, InstanceV1_1};

// Extension Objects
use ash::vk::{
    AccelerationStructureKHR, DeferredOperationKHR, GeometryTypeKHR,
    PhysicalDeviceBufferDeviceAddressFeaturesEXT, PhysicalDeviceFeatures2KHR,
    PhysicalDeviceRayTracingPipelineFeaturesKHR, PhysicalDeviceRayTracingPipelinePropertiesKHR,
    PhysicalDeviceVulkan12Features, RayTracingPipelineCreateInfoKHR,
    RayTracingShaderGroupCreateInfoKHR, RayTracingShaderGroupTypeKHR, SHADER_UNUSED_KHR,
};
// Core objects
use ash::vk::{
    BufferUsageFlags, CommandBufferAllocateInfo, CommandPool, CommandPoolCreateInfo,
    DescriptorBindingFlagsEXT, DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize,
    DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding,
    DescriptorSetLayoutBindingFlagsCreateInfoEXT, DescriptorSetLayoutCreateInfo, DescriptorType,
    DeviceCreateInfo, DeviceQueueCreateInfo, Format, ImageUsageFlags, ImageView,
    MemoryPropertyFlags, PhysicalDevice, PhysicalDeviceAccelerationStructureFeaturesKHR,
    PhysicalDeviceMemoryProperties, PhysicalDeviceMemoryProperties2, PhysicalDeviceProperties2,
    Pipeline, PipelineCache, PipelineLayout, PipelineLayoutCreateInfo,
    PipelineShaderStageCreateInfo, PushConstantRange, Queue, QueueFlags, ShaderModuleCreateInfo,
    ShaderStageFlags,
};

use ash::{Device, Instance};

pub struct Renderer {
    physical_device_memory_properties: PhysicalDeviceMemoryProperties2,
    context: RtxContext,
    queue_family_index: u32,
    descriptor_sets: DescriptorSet,
    pipeline_properties: PhysicalDeviceRayTracingPipelinePropertiesKHR,
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    descriptor_pool: DescriptorPool,
    descriptor_set_layouts: Vec<DescriptorSetLayout>,
    accumulation_image: Option<Image2DResource>,
    output_image: Option<Image2DResource>,
    output_image_view: ImageView,
    bottom_level_acceleration_structures: Vec<AccelerationStructureKHR>,
    top_level_acceleration_structure: AccelerationStructureKHR,
    shader_binding_table: Option<BufferResource>,
}

impl Renderer {
    pub fn initialize(&mut self, width: u32, height: u32) {
        self.create_images_and_views(width, height);
    }

    pub fn build(&mut self, vertices: &[f32], indices: &[u32]) {
        let vertex_buffer = BufferResource::new(
            &self.physical_device_memory_properties.memory_properties,
            &self.context.device(),
            (vertices.len() * 12) as u64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::VERTEX_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        let index_buffer = BufferResource::new(
            &self.physical_device_memory_properties.memory_properties,
            &self.context.device(),
            (indices.len() * 4) as u64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        let blases = [BottomLevelAccelerationStructure::new(
            &self.context,
            &vertex_buffer,
            vertices.len() as u32,
            0,
            &index_buffer,
            indices.len() as u32,
            0,
        )];

        let instances = [GeometryInstance::new(0, 0xff, 0, 0, 0)];

        let tlas = TopLevelAccelerationStructure::new(&self.context, &blases, &instances);
    }
}

impl Renderer {
    pub fn new(
        instance: &Instance,
        gpu_and_queue_family_index: Option<(PhysicalDevice, u32)>,
    ) -> Self {
        unsafe {
            let (gpu, queue_family_index) = {
                if let Some((gpu, queue_family_index)) = gpu_and_queue_family_index {
                    (gpu, queue_family_index)
                } else {
                    let pdevices = instance
                        .enumerate_physical_devices()
                        .expect("Physical device error");
                    pdevices
                        .iter()
                        .map(|pdevice| {
                            instance
                                .get_physical_device_queue_family_properties(*pdevice)
                                .iter()
                                .enumerate()
                                .filter_map(|(index, ref info)| {
                                    let supports_graphics =
                                        info.queue_flags.contains(QueueFlags::GRAPHICS);
                                    if supports_graphics {
                                        Some((*pdevice, index as u32))
                                    } else {
                                        None
                                    }
                                })
                                .next()
                        })
                        .filter_map(|v| v)
                        .next()
                        .expect("Couldn't find suitable device.")
                }
            };
            let priorities = [1.0];
            let queue_info = [DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index as u32)
                .queue_priorities(&priorities)
                .build()];
            let device_extension_names_raw = [
                RayTracingPipeline::name().as_ptr(),
                DeferredHostOperations::name().as_ptr(),
                AccelerationStructure::name().as_ptr(),
            ];
            let mut pipeline_properties = PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
            let mut properties =
                PhysicalDeviceProperties2::builder().push_next(&mut pipeline_properties);
            instance.get_physical_device_properties2(gpu, &mut properties);
            let mut rt_features = PhysicalDeviceRayTracingPipelineFeaturesKHR {
                ray_tracing_pipeline: 1,
                ..Default::default()
            };
            let mut address_features = PhysicalDeviceVulkan12Features::builder()
                .buffer_device_address(true)
                .build();
            let mut acc_features = PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
                .acceleration_structure(true)
                .build();
            let mut features2 = PhysicalDeviceFeatures2KHR::default();
            instance.get_physical_device_features2(gpu, &mut features2);
            let device_create_info = DeviceCreateInfo::builder()
                .queue_create_infos(&queue_info)
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features2.features)
                .push_next(&mut rt_features)
                .push_next(&mut address_features)
                .push_next(&mut acc_features);
            let device = instance
                .create_device(gpu, &device_create_info, None)
                .expect("Failed raytracing device context creation");
            let queue = device.get_device_queue(queue_family_index as u32, 0);
            let mut physical_device_memory_properties = PhysicalDeviceMemoryProperties2::default();
            instance.get_physical_device_memory_properties2(
                gpu,
                &mut physical_device_memory_properties,
            );

            let context = RtxContext::new(
                instance,
                &device,
                &queue,
                queue_family_index as u32,
                &physical_device_memory_properties,
            );

            let mut result = Self {
                physical_device_memory_properties,
                context,
                queue_family_index: queue_family_index as u32,
                descriptor_sets: DescriptorSet::null(),
                pipeline: Pipeline::null(),
                pipeline_properties,
                pipeline_layout: PipelineLayout::null(),
                descriptor_pool: DescriptorPool::null(),
                descriptor_set_layouts: Vec::new(),
                accumulation_image: None,
                output_image: None,
                output_image_view: ImageView::null(),
                top_level_acceleration_structure: AccelerationStructureKHR::null(),
                bottom_level_acceleration_structures: Vec::new(),
                shader_binding_table: None,
            };
            result.create_descriptor_pool();
            result.create_descriptor_set_layout();
            result.create_descriptor_set();
            result.create_pipeline_layout();
            result.load_shaders_and_pipeline();
            result.create_shader_binding_table();
            result
        }
    }

    fn create_descriptor_pool(&mut self) {
        unsafe {
            let sizes = [
                DescriptorPoolSize {
                    ty: DescriptorType::ACCELERATION_STRUCTURE_KHR,
                    descriptor_count: 1,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::STORAGE_IMAGE,
                    descriptor_count: 2,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 4,
                },
            ];
            let descriptor_pool_create_info = DescriptorPoolCreateInfo::builder()
                .max_sets(2)
                .pool_sizes(&sizes);
            self.descriptor_pool = self
                .context
                .device()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Descriptor pool creation failed");
        }
    }

    fn create_descriptor_set_layout(&mut self) {
        unsafe {
            let bindings_ray_gen = [
                // acceleration structure
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(0)
                    .build(),
                // final image
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_IMAGE)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(1)
                    .build(),
                // accumulation image
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_IMAGE)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(2)
                    .build(),
                // Camera
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(3)
                    .build(),
            ];

            let bindings_closest_hit = [
                // position buffer
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_BUFFER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(0)
                    .build(),
                // index buffer
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_BUFFER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(1)
                    .build(),
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_BUFFER)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(2)
                    .build(),
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_BUFFER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(3)
                    .build(),
            ];

            let mut binding_flags_ray_gen = DescriptorSetLayoutBindingFlagsCreateInfoEXT::builder()
                .binding_flags(&[
                    DescriptorBindingFlagsEXT::empty(),
                    DescriptorBindingFlagsEXT::empty(),
                    DescriptorBindingFlagsEXT::empty(),
                    DescriptorBindingFlagsEXT::empty(),
                ])
                .build();

            let mut binding_flags_closest_hit =
                DescriptorSetLayoutBindingFlagsCreateInfoEXT::builder()
                    .binding_flags(&[
                        DescriptorBindingFlagsEXT::empty(),
                        DescriptorBindingFlagsEXT::empty(),
                        DescriptorBindingFlagsEXT::empty(),
                        DescriptorBindingFlagsEXT::empty(),
                    ])
                    .build();

            let layout_info_ray_gen = DescriptorSetLayoutCreateInfo::builder()
                .bindings(&bindings_ray_gen)
                .push_next(&mut binding_flags_ray_gen);

            let layout_info_closest_hit = DescriptorSetLayoutCreateInfo::builder()
                .bindings(&bindings_closest_hit)
                .push_next(&mut binding_flags_closest_hit);

            self.descriptor_set_layouts = vec![
                self.context
                    .device()
                    .create_descriptor_set_layout(&layout_info_ray_gen, None)
                    .expect("Descriptor set layout creation failed"),
                self.context
                    .device()
                    .create_descriptor_set_layout(&layout_info_closest_hit, None)
                    .expect("Descriptor set layout creation failed"),
            ]
        }
    }

    fn create_pipeline_layout(&mut self) {
        unsafe {
            let push_constant_ranges = [PushConstantRange::builder()
                .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                .size(24)
                .build()];
            self.pipeline_layout = self
                .context
                .device()
                .create_pipeline_layout(
                    &PipelineLayoutCreateInfo::builder()
                        .set_layouts(&self.descriptor_set_layouts)
                        .push_constant_ranges(&push_constant_ranges),
                    None,
                )
                .expect("Pipeline layout creation failed");
        }
    }

    fn create_descriptor_set(&mut self) {
        unsafe {
            let descriptor_set_create_info = DescriptorSetAllocateInfo::builder()
                .set_layouts(&self.descriptor_set_layouts)
                .descriptor_pool(self.descriptor_pool);

            self.descriptor_sets = self
                .context
                .device()
                .allocate_descriptor_sets(&descriptor_set_create_info)
                .expect("Descriptor set allocation failed")[0];
        }
    }

    fn load_shaders_and_pipeline(&mut self) {
        unsafe {
            let code = load_spirv("shaders/rgen.rgen.spv");
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let gen = self
                .context
                .device()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray generation shader compilation failed");
            let code = load_spirv("shaders/closesthit.rchit.spv");
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let chit = self
                .context
                .device()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray closest hit shader compilation failed");
            let code = load_spirv("shaders/miss.rmiss.spv");
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let miss = self
                .context
                .device()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray miss shader compilation failed");

            let shader_groups = vec![
                // group0 = [ raygen ]
                RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(RayTracingShaderGroupTypeKHR::GENERAL)
                    .general_shader(0)
                    .closest_hit_shader(SHADER_UNUSED_KHR)
                    .any_hit_shader(SHADER_UNUSED_KHR)
                    .intersection_shader(SHADER_UNUSED_KHR)
                    .build(),
                // group1 = [ chit ]
                RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP)
                    .general_shader(SHADER_UNUSED_KHR)
                    .closest_hit_shader(1)
                    .any_hit_shader(SHADER_UNUSED_KHR)
                    .intersection_shader(SHADER_UNUSED_KHR)
                    .build(),
                // group2 = [ miss ]
                RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(RayTracingShaderGroupTypeKHR::GENERAL)
                    .general_shader(2)
                    .closest_hit_shader(SHADER_UNUSED_KHR)
                    .any_hit_shader(SHADER_UNUSED_KHR)
                    .intersection_shader(SHADER_UNUSED_KHR)
                    .build(),
            ];

            let shader_stages = vec![
                PipelineShaderStageCreateInfo::builder()
                    .stage(ShaderStageFlags::RAYGEN_KHR)
                    .module(gen)
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap())
                    .build(),
                PipelineShaderStageCreateInfo::builder()
                    .stage(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .module(chit)
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap())
                    .build(),
                PipelineShaderStageCreateInfo::builder()
                    .stage(ShaderStageFlags::MISS_KHR)
                    .module(miss)
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap())
                    .build(),
            ];

            let infos = [RayTracingPipelineCreateInfoKHR::builder()
                .stages(&shader_stages)
                .groups(&shader_groups)
                .max_pipeline_ray_recursion_depth(1)
                .layout(self.pipeline_layout)
                .build()];

            self.pipeline = self
                .context
                .pipeline_ext()
                .create_ray_tracing_pipelines(
                    DeferredOperationKHR::null(),
                    PipelineCache::null(),
                    &infos,
                    None,
                )
                .expect("Raytracing pipeline creation failed")[0];
        }
    }

    fn create_shader_binding_table(&mut self) {
        unsafe {
            let group_count = 3;
            let table_size =
                (self.pipeline_properties.shader_group_handle_size * group_count) as usize;
            let table_data: Vec<u8> = self
                .context
                .pipeline_ext()
                .get_ray_tracing_shader_group_handles(self.pipeline, 0, group_count, table_size)
                .expect("Get raytracing shader group handles failed");

            self.shader_binding_table = Some(BufferResource::new(
                &self.physical_device_memory_properties.memory_properties,
                &self.context.device(),
                table_size as u64,
                MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
                BufferUsageFlags::TRANSFER_SRC,
            ));

            self.shader_binding_table
                .as_mut()
                .unwrap()
                .copy_to(&table_data);
        }
    }

    fn create_images_and_views(&mut self, width: u32, height: u32) {
        self.output_image = Some(Image2DResource::new(
            &self.physical_device_memory_properties.memory_properties,
            &self.context.device(),
            width,
            height,
            Format::R8G8B8A8_UNORM,
            ImageUsageFlags::STORAGE | ImageUsageFlags::SAMPLED,
            MemoryPropertyFlags::DEVICE_LOCAL,
        ));

        self.accumulation_image = Some(Image2DResource::new(
            &self.physical_device_memory_properties.memory_properties,
            &self.context.device(),
            width,
            height,
            Format::R32G32B32A32_SFLOAT,
            ImageUsageFlags::STORAGE,
            MemoryPropertyFlags::DEVICE_LOCAL,
        ));
    }
}
