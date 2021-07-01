use crate::context::RtxContext;
use crate::geometry::{
    BottomLevelAccelerationStructure, GeometryBuffer, GeometryBufferView, GeometryInstance,
    TopLevelAccelerationStructure, Vertex,
};
use glm::Vec3;

use vk_utils::buffer_resource::BufferResource;
use vk_utils::device_context::DeviceContext;
use vk_utils::gpu::Gpu;
use vk_utils::image_resource::Image2DResource;
use vk_utils::shader_library::load_spirv;

// Version traits
use ash::version::{DeviceV1_0, InstanceV1_1};

use ash::extensions::khr::{AccelerationStructure, RayTracingPipeline};

// Extension Objects
use ash::vk::{
    DeferredOperationKHR, DeviceSize, PhysicalDeviceFeatures2KHR,
    PhysicalDeviceRayTracingPipelineFeaturesKHR, PhysicalDeviceRayTracingPipelinePropertiesKHR,
    PhysicalDeviceVulkan12Features, RayTracingPipelineCreateInfoKHR,
    RayTracingShaderGroupCreateInfoKHR, RayTracingShaderGroupTypeKHR,
    StridedDeviceAddressRegionKHR, WriteDescriptorSetAccelerationStructureKHR, SHADER_UNUSED_KHR,
};
// Core objects
use ash::vk::{
    BufferUsageFlags, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPool,
    DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo,
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType,
    Format, ImageAspectFlags, ImageLayout, ImageSubresourceRange, ImageUsageFlags, ImageView,
    ImageViewCreateInfo, ImageViewType, MemoryPropertyFlags,
    PhysicalDeviceAccelerationStructureFeaturesKHR, PhysicalDeviceMemoryProperties2, Pipeline,
    PipelineBindPoint, PipelineCache, PipelineLayout, PipelineLayoutCreateInfo,
    PipelineShaderStageCreateInfo, ShaderModuleCreateInfo, ShaderStageFlags, WriteDescriptorSet,
};

pub struct Renderer {
    device: DeviceContext,
    rtx: RtxContext,
    descriptor_sets: Vec<DescriptorSet>,
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    descriptor_pool: DescriptorPool,
    descriptor_set_layouts: Vec<DescriptorSetLayout>,
    accumulation_image: Option<Image2DResource>,
    output_image: Option<Image2DResource>,
    output_image_view: ImageView,
    vertex_buffer: Option<BufferResource>,
    index_buffer: Option<BufferResource>,
    camera_buffer: BufferResource,
    bottom_level_acceleration_structures: Vec<BottomLevelAccelerationStructure>,
    top_level_acceleration_structure: Option<TopLevelAccelerationStructure>,
    shader_binding_table: Option<BufferResource>,
    stride_addresses: Vec<StridedDeviceAddressRegionKHR>,

    output_width: u32,
    output_height: u32,
}

impl Renderer {
    pub fn initialize(&mut self, width: u32, height: u32) {
        self.create_images_and_views(width, height);
        self.update_image_descriptors();
        self.update_camera_descriptors();
        self.output_width = width;
        self.output_height = height;
    }

    pub fn download_image(&self) -> BufferResource {
        self.device.wait();
        let buffer = self.device.buffer(
            (self.output_width * self.output_height * 4) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::TRANSFER_DST,
        );

        if let Some(queue) = self.device.graphics_queue() {
            queue.begin(|command_buffer| {
                command_buffer
                    .copy_image_2d_to_buffer(&self.output_image.as_ref().unwrap(), &buffer);
                command_buffer
            });
        }
        buffer
    }

    pub fn render(&mut self) -> ImageView {
        unsafe {
            if let Some(queue) = self.device.graphics_queue() {
                queue.begin(|command_buffer| {
                    if let Some(image) = self.output_image.as_ref() {
                        command_buffer.image_transition(image, ImageLayout::GENERAL);
                        //command_buffer.clear_image_2d(image, 1., 1., 1., 1.);
                    } else {
                        panic!()
                    }

                    command_buffer.bind_descriptor_sets(
                        &self.pipeline_layout,
                        PipelineBindPoint::RAY_TRACING_KHR,
                        &self.descriptor_sets,
                    );

                    self.device.vk_device().cmd_bind_pipeline(
                        *command_buffer.native_handle(),
                        PipelineBindPoint::RAY_TRACING_KHR,
                        self.pipeline,
                    );

                    self.rtx.pipeline_ext().cmd_trace_rays(
                        *command_buffer.native_handle(),
                        &self.stride_addresses[0],
                        &self.stride_addresses[1],
                        &self.stride_addresses[2],
                        &StridedDeviceAddressRegionKHR::default(),
                        self.output_width,
                        self.output_height,
                        1,
                    );

                    command_buffer
                });
            }

            self.output_image.as_mut().unwrap().layout = ImageLayout::GENERAL;
            self.output_image_view
        }
    }

    pub fn build(&mut self, geometry: &GeometryBuffer, views: &[GeometryBufferView]) {
        self.vertex_buffer = Some(self.device.buffer(
            (geometry.vertices().len() * 4 * 3) as u64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::VERTEX_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        ));

        self.index_buffer = Some(self.device.buffer(
            (geometry.indices().len() * 4) as u64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        ));

        self.bottom_level_acceleration_structures = views
            .iter()
            .map(|view| {
                BottomLevelAccelerationStructure::new(
                    &self.device,
                    &self.rtx,
                    &self.vertex_buffer.as_ref().unwrap(),
                    view.vertex_count(),
                    view.vertex_offset(),
                    &self.index_buffer.as_ref().unwrap(),
                    view.index_count(),
                    view.index_offset(),
                )
            })
            .collect();

        let instances = [GeometryInstance::new(
            0,
            0xff,
            0,
            0,
            self.bottom_level_acceleration_structures[0].address(),
        )];

        let tlas = TopLevelAccelerationStructure::new(
            &self.device,
            &self.rtx,
            &self.bottom_level_acceleration_structures,
            &instances,
        );
        self.top_level_acceleration_structure = Some(tlas);
        self.update_acceleration_structure_descriptors();
    }
}

impl Renderer {
    pub fn new(gpu: &Gpu) -> Self {
        unsafe {
            let device = {
                let mut rt_features = PhysicalDeviceRayTracingPipelineFeaturesKHR::builder()
                    .ray_tracing_pipeline(true);
                let mut address_features =
                    PhysicalDeviceVulkan12Features::builder().buffer_device_address(true);
                let mut acc_features = PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
                    .acceleration_structure(true);
                let mut features2 = PhysicalDeviceFeatures2KHR::default();
                gpu.vulkan()
                    .vk_instance()
                    .get_physical_device_features2(*gpu.vk_physical_device(), &mut features2);

                gpu.device_context(
                    &[
                        ash::extensions::khr::RayTracingPipeline::name(),
                        ash::extensions::khr::AccelerationStructure::name(),
                        ash::extensions::khr::DeferredHostOperations::name(),
                    ],
                    |builder| {
                        builder
                            .push_next(&mut address_features)
                            .push_next(&mut rt_features)
                            .push_next(&mut acc_features)
                            .enabled_features(&features2.features)
                    },
                )
            };

            let mut physical_device_memory_properties = PhysicalDeviceMemoryProperties2::default();
            gpu.vulkan()
                .vk_instance()
                .get_physical_device_memory_properties2(
                    *gpu.vk_physical_device(),
                    &mut physical_device_memory_properties,
                );

            let acceleration_structure_ext =
                AccelerationStructure::new(gpu.vulkan().vk_instance(), device.vk_device());
            let ray_tracing_pipeline_ext =
                RayTracingPipeline::new(gpu.vulkan().vk_instance(), device.vk_device());

            let mut pipeline_properties = PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
            let _properties =
                gpu.extension_properties(|builder| builder.push_next(&mut pipeline_properties));

            let rtx = RtxContext::new(
                acceleration_structure_ext,
                ray_tracing_pipeline_ext,
                physical_device_memory_properties,
                pipeline_properties,
            );

            let camera_buffer = device.buffer(
                96,
                MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
                BufferUsageFlags::UNIFORM_BUFFER,
            );

            let mut result = Self {
                device,
                rtx,
                descriptor_sets: Vec::new(),
                pipeline: Pipeline::null(),
                pipeline_layout: PipelineLayout::null(),
                descriptor_pool: DescriptorPool::null(),
                descriptor_set_layouts: Vec::new(),
                accumulation_image: None,
                output_image: None,
                output_image_view: ImageView::null(),
                vertex_buffer: None,
                index_buffer: None,
                camera_buffer,
                top_level_acceleration_structure: None,
                bottom_level_acceleration_structures: Vec::new(),
                shader_binding_table: None,
                stride_addresses: Vec::new(),
                output_width: 0,
                output_height: 0,
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

    pub fn set_camera(&mut self, origin: &Vec3, target: &Vec3) {
        let view_matrix = glm::ext::look_at(*origin, *target, glm::vec3(0., 1., 0.));
        let view_matrix = glm::inverse(&view_matrix);

        let projection_matrix = glm::ext::perspective(
            65.,
            self.output_width as f32 / self.output_height as f32,
            0.1,
            100.,
        );
        let projection_matrix = glm::inverse(&projection_matrix);
        self.camera_buffer
            .copy_to(&[view_matrix, projection_matrix]);
    }

    fn create_descriptor_pool(&mut self) {
        unsafe {
            let sizes = [
                DescriptorPoolSize {
                    ty: DescriptorType::ACCELERATION_STRUCTURE_KHR,
                    descriptor_count: 1,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::STORAGE_IMAGE,
                    descriptor_count: 1,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                },
            ];
            let descriptor_pool_create_info = DescriptorPoolCreateInfo::builder()
                .max_sets(2)
                .pool_sizes(&sizes);
            self.descriptor_pool = self
                .device
                .vk_device()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Descriptor pool creation failed");
        }
    }

    fn create_descriptor_set_layout(&mut self) {
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

            let set_0 = DescriptorSetLayoutCreateInfo::builder().bindings(&set_0_bindings);

            let set_1_bindings = [*DescriptorSetLayoutBinding::builder()
                .descriptor_count(1)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                .binding(0)];

            let set_1 = DescriptorSetLayoutCreateInfo::builder().bindings(&set_1_bindings);

            self.descriptor_set_layouts = vec![
                self.device
                    .vk_device()
                    .create_descriptor_set_layout(&set_0, None)
                    .expect("Descriptor set layout creation failed"),
                self.device
                    .vk_device()
                    .create_descriptor_set_layout(&set_1, None)
                    .expect("Descriptor set layout creation failed"),
            ]
        }
    }

    fn create_pipeline_layout(&mut self) {
        unsafe {
            self.pipeline_layout = self
                .device
                .vk_device()
                .create_pipeline_layout(
                    &PipelineLayoutCreateInfo::builder().set_layouts(&self.descriptor_set_layouts),
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
                .device
                .vk_device()
                .allocate_descriptor_sets(&descriptor_set_create_info)
                .expect("Descriptor set allocation failed");
        }
    }

    fn load_shaders_and_pipeline(&mut self) {
        unsafe {
            let code = load_spirv("shaders/simple_pipeline/ray_gen.rgen.spv");
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let gen = self
                .device
                .vk_device()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray generation shader compilation failed");
            let code = load_spirv("shaders/simple_pipeline/closest_hit.rchit.spv");
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let chit = self
                .device
                .vk_device()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray closest hit shader compilation failed");
            let code = load_spirv("shaders/simple_pipeline/ray_miss.rmiss.spv");
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let miss = self
                .device
                .vk_device()
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
                .rtx
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
            let properties = self.rtx.pipeline_properties();
            let aligned_group_size = properties.shader_group_handle_size
                + (properties.shader_group_base_alignment - properties.shader_group_handle_size);
            let table_size = (aligned_group_size * group_count) as usize;
            let table_data: Vec<u8> = self
                .rtx
                .pipeline_ext()
                .get_ray_tracing_shader_group_handles(self.pipeline, 0, group_count, table_size)
                .expect("Get raytracing shader group handles failed");

            self.shader_binding_table = Some(self.device.buffer(
                table_size as u64,
                MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
                BufferUsageFlags::TRANSFER_SRC
                    | BufferUsageFlags::SHADER_BINDING_TABLE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            ));

            self.shader_binding_table.as_mut().unwrap().copy_aligned_to(
                &table_data,
                Some(properties.shader_group_handle_size as usize),
                aligned_group_size as usize,
            );

            let ray_gen_address = StridedDeviceAddressRegionKHR::builder()
                .size(self.rtx.pipeline_properties().shader_group_base_alignment as DeviceSize)
                .stride(self.rtx.pipeline_properties().shader_group_base_alignment as DeviceSize)
                .device_address(self.shader_binding_table.as_ref().unwrap().device_address())
                .build();

            let closest_hit_address = StridedDeviceAddressRegionKHR::builder()
                .size(self.rtx.pipeline_properties().shader_group_base_alignment as DeviceSize)
                .stride(self.rtx.pipeline_properties().shader_group_base_alignment as DeviceSize)
                .device_address(
                    self.shader_binding_table.as_ref().unwrap().device_address()
                        + self.rtx.pipeline_properties().shader_group_base_alignment as DeviceSize
                            * 2,
                )
                .build();

            let miss_address = StridedDeviceAddressRegionKHR::builder()
                .size(self.rtx.pipeline_properties().shader_group_base_alignment as DeviceSize)
                .stride(self.rtx.pipeline_properties().shader_group_base_alignment as DeviceSize)
                .device_address(
                    self.shader_binding_table.as_ref().unwrap().device_address()
                        + self.rtx.pipeline_properties().shader_group_base_alignment as DeviceSize,
                )
                .build();

            self.stride_addresses = vec![ray_gen_address, miss_address, closest_hit_address];
        }
    }

    fn create_images_and_views(&mut self, width: u32, height: u32) {
        self.output_image = Some(self.device.image_2d(
            width,
            height,
            Format::R8G8B8A8_UNORM,
            MemoryPropertyFlags::DEVICE_LOCAL,
            ImageUsageFlags::STORAGE
                | ImageUsageFlags::SAMPLED
                | ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST,
        ));

        self.accumulation_image = Some(self.device.image_2d(
            width,
            height,
            Format::R32G32B32A32_SFLOAT,
            MemoryPropertyFlags::DEVICE_LOCAL,
            ImageUsageFlags::STORAGE,
        ));

        let view_info = ImageViewCreateInfo::builder()
            .format(Format::R8G8B8A8_UNORM)
            .subresource_range(
                ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1)
                    .build(),
            )
            .view_type(ImageViewType::TYPE_2D)
            .image(*self.output_image.as_ref().unwrap().vk_image());

        unsafe {
            self.output_image_view = self
                .device
                .vk_device()
                .create_image_view(&view_info, None)
                .expect("Image View creation failed");
        }
    }

    fn update_image_descriptors(&mut self) {
        let image_writes = [DescriptorImageInfo::builder()
            .image_view(self.output_image_view)
            .image_layout(ImageLayout::GENERAL)
            .build()];
        let writes = [WriteDescriptorSet::builder()
            .image_info(&image_writes)
            .dst_set(self.descriptor_sets[0])
            .dst_binding(1)
            .descriptor_type(DescriptorType::STORAGE_IMAGE)
            .build()];

        unsafe {
            self.device.vk_device().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_acceleration_structure_descriptors(&mut self) {
        let structures = [self
            .top_level_acceleration_structure
            .as_ref()
            .unwrap()
            .acceleration_structure
            .clone()];
        let mut acc_write = WriteDescriptorSetAccelerationStructureKHR::builder()
            .acceleration_structures(&structures)
            .build();

        let mut writes = [WriteDescriptorSet::builder()
            .push_next(&mut acc_write)
            .dst_set(self.descriptor_sets[0])
            .dst_binding(0)
            .descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .build()];

        writes[0].descriptor_count = 1;

        unsafe {
            self.device.vk_device().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_camera_descriptors(&mut self) {
        let buffer_write = [*DescriptorBufferInfo::builder()
            .range(96)
            .buffer(self.camera_buffer.buffer)];

        let mut writes = [WriteDescriptorSet::builder()
            .buffer_info(&buffer_write)
            .dst_set(self.descriptor_sets[1])
            .dst_binding(0)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .build()];

        writes[0].descriptor_count = 1;

        unsafe {
            self.device.vk_device().update_descriptor_sets(&writes, &[]);
        }
    }
}
