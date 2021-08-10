use crate::context::RtxContext;
use crate::descriptor_sets::RTXDescriptorSets;
use crate::scene::Scene;
use crate::scene_data::SceneData;
use nalgebra_glm::Vec3;

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
    DeferredOperationKHR, PhysicalDeviceRayTracingPipelinePropertiesKHR,
    RayTracingPipelineCreateInfoKHR, RayTracingShaderGroupCreateInfoKHR,
    RayTracingShaderGroupTypeKHR, StridedDeviceAddressRegionKHR,
    WriteDescriptorSetAccelerationStructureKHR, SHADER_UNUSED_KHR,
};
// Core objects
use ash::vk::{
    BufferUsageFlags, DescriptorBufferInfo, DescriptorImageInfo, DescriptorType, Format,
    ImageAspectFlags, ImageLayout, ImageSubresourceRange, ImageUsageFlags, ImageView,
    ImageViewCreateInfo, ImageViewType, MemoryPropertyFlags, PhysicalDeviceMemoryProperties2,
    Pipeline, PipelineBindPoint, PipelineCache, PipelineShaderStageCreateInfo,
    ShaderModuleCreateInfo, ShaderStageFlags, WriteDescriptorSet,
};

pub struct Renderer {
    rtx: RtxContext,
    pipeline: Pipeline,
    accumulation_image: Option<Image2DResource>,
    output_image: Option<Image2DResource>,
    output_image_view: ImageView,
    camera_buffer: BufferResource,
    shader_binding_table: Option<BufferResource>,
    stride_addresses: Vec<StridedDeviceAddressRegionKHR>,

    scene_data: Option<SceneData>,
    descriptor_sets: RTXDescriptorSets,

    output_width: u32,
    output_height: u32,
}

unsafe impl Send for Renderer {}

impl Renderer {
    pub fn download_image(&self, device: &DeviceContext) -> BufferResource {
        device.wait();
        let buffer = device.buffer(
            (self.output_width * self.output_height * 4) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::TRANSFER_DST,
        );

        if let Some(queue) = device.graphics_queue() {
            queue.begin(|command_buffer| {
                command_buffer
                    .copy_image_2d_to_buffer(&self.output_image.as_ref().unwrap(), &buffer);
                command_buffer
            });
        }
        buffer
    }

    pub fn render(&mut self, device: &DeviceContext) -> &ImageView {
        unsafe {
            if let Some(_scene) = self.scene_data.as_ref() {
                if let Some(queue) = device.graphics_queue() {
                    queue.begin(|command_buffer| {
                        if let Some(image) = self.output_image.as_ref() {
                            command_buffer
                                .color_image_resource_transition(image, ImageLayout::GENERAL);
                            self.output_image.as_mut().unwrap().layout = ImageLayout::GENERAL;
                        } else {
                            panic!()
                        }
                        command_buffer.bind_descriptor_sets(
                            &self.descriptor_sets.pipeline_layout,
                            PipelineBindPoint::RAY_TRACING_KHR,
                            &self.descriptor_sets.descriptor_sets,
                        );
                        device.vk_device().cmd_bind_pipeline(
                            *command_buffer.native_handle(),
                            PipelineBindPoint::RAY_TRACING_KHR,
                            self.pipeline,
                        );
                        self.rtx.pipeline_ext().cmd_trace_rays(
                            *command_buffer.native_handle(),
                            &self.stride_addresses[0],
                            &self.stride_addresses[1],
                            &self.stride_addresses[2],
                            &self.stride_addresses[3],
                            self.output_width,
                            self.output_height,
                            1,
                        );
                        if let Some(image) = self.output_image.as_ref() {
                            command_buffer.color_image_resource_transition(
                                image,
                                ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                            );
                        } else {
                            panic!()
                        }
                        command_buffer
                    });
                }
                self.output_image.as_mut().unwrap().layout = ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                &self.output_image_view
            } else {
                panic!("No scene")
            }
        }
    }

    pub fn build(&mut self, device: &DeviceContext, scene: &Scene) {
        self.scene_data = Some(SceneData::new(&device, &self.rtx, scene));

        self.descriptor_sets
            .update_scene_descriptors(&device, self.scene_data.as_ref().unwrap());
        self.update_acceleration_structure_descriptors(device);
    }
}

impl Renderer {
    pub fn new(gpu: &Gpu, device: &DeviceContext, width: u32, height: u32) -> Self {
        let mut physical_device_memory_properties = PhysicalDeviceMemoryProperties2::default();
        unsafe {
            gpu.vulkan()
                .vk_instance()
                .get_physical_device_memory_properties2(
                    *gpu.vk_physical_device(),
                    &mut physical_device_memory_properties,
                );
        }

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
            128,
            MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::UNIFORM_BUFFER,
        );

        let descriptor_sets = RTXDescriptorSets::new(&device);

        let mut result = Self {
            rtx,
            pipeline: Pipeline::null(),
            accumulation_image: None,
            output_image: None,
            output_image_view: ImageView::null(),
            camera_buffer,
            shader_binding_table: None,
            stride_addresses: Vec::new(),

            descriptor_sets,
            scene_data: None,

            output_width: 0,
            output_height: 0,
        };

        result.load_shaders_and_pipeline(device);
        result.create_shader_binding_table(device);
        result.create_images_and_views(device, width, height);
        result.update_image_descriptors(device);
        result.update_camera_descriptors(device);
        result.output_width = width;
        result.output_height = height;
        result
    }

    pub fn set_camera(&mut self, origin: &Vec3, target: &Vec3) {
        let view_matrix = glm::look_at_rh(origin, target, &glm::vec3(0., 1., 0.));
        let view_matrix = glm::inverse(&view_matrix);

        let projection_matrix = glm::perspective_rh(
            self.output_width as f32 / self.output_height as f32,
            1.134,
            0.1,
            1000.,
        );

        let projection_matrix = glm::inverse(&projection_matrix);
        self.camera_buffer
            .copy_to(&[view_matrix, projection_matrix]);
    }

    fn load_shaders_and_pipeline(&mut self, device: &DeviceContext) {
        unsafe {
            let code = load_spirv(
                "C:/Users/danny/Documents/code/tracey/shaders/simple_pipeline/ray_gen.rgen.spv",
            );
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let gen = device
                .vk_device()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray generation shader compilation failed");
            let code =
                load_spirv("C:/Users/danny/Documents/code/tracey/shaders/simple_pipeline/closest_hit.rchit.spv");
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let chit = device
                .vk_device()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray closest hit shader compilation failed");
            let code = load_spirv(
                "C:/Users/danny/Documents/code/tracey/shaders/simple_pipeline/ray_miss.rmiss.spv",
            );
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let miss = device
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
                .layout(self.descriptor_sets.pipeline_layout)
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

    fn create_shader_binding_table(&mut self, device: &DeviceContext) {
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

            self.shader_binding_table = Some(device.buffer(
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

            let ray_gen_address = *StridedDeviceAddressRegionKHR::builder()
                .size(aligned_group_size.into())
                .stride(aligned_group_size.into())
                .device_address(self.shader_binding_table.as_ref().unwrap().device_address());

            let closest_hit_address = *StridedDeviceAddressRegionKHR::builder()
                .size(aligned_group_size.into())
                .stride(aligned_group_size.into())
                .device_address(
                    self.shader_binding_table.as_ref().unwrap().device_address()
                        + aligned_group_size as u64,
                );

            let miss_address = *StridedDeviceAddressRegionKHR::builder()
                .size(aligned_group_size.into())
                .stride(aligned_group_size.into())
                .device_address(
                    self.shader_binding_table.as_ref().unwrap().device_address()
                        + aligned_group_size as u64 * 2,
                );

            let callable_address = *StridedDeviceAddressRegionKHR::builder()
                .size(0)
                .stride(0)
                .device_address(0);

            self.stride_addresses = vec![
                ray_gen_address,
                miss_address,
                closest_hit_address,
                callable_address,
            ];
        }
    }

    fn create_images_and_views(&mut self, device: &DeviceContext, width: u32, height: u32) {
        self.output_image = Some(device.image_2d(
            width,
            height,
            Format::R8G8B8A8_UNORM,
            MemoryPropertyFlags::DEVICE_LOCAL,
            ImageUsageFlags::STORAGE
                | ImageUsageFlags::SAMPLED
                | ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST,
        ));

        self.accumulation_image = Some(device.image_2d(
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
            self.output_image_view = device
                .vk_device()
                .create_image_view(&view_info, None)
                .expect("Image View creation failed");
        }
    }

    fn update_image_descriptors(&mut self, device: &DeviceContext) {
        let image_writes = [*DescriptorImageInfo::builder()
            .image_view(self.output_image_view)
            .image_layout(ImageLayout::GENERAL)];
        let writes = [*WriteDescriptorSet::builder()
            .image_info(&image_writes)
            .dst_set(self.descriptor_sets.descriptor_sets[0])
            .dst_binding(1)
            .descriptor_type(DescriptorType::STORAGE_IMAGE)];

        unsafe {
            device.vk_device().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_acceleration_structure_descriptors(&mut self, device: &DeviceContext) {
        let structures = [self
            .scene_data
            .as_ref()
            .unwrap()
            .top_level_acceleration_structure
            .acceleration_structure
            .clone()];
        let mut acc_write = *WriteDescriptorSetAccelerationStructureKHR::builder()
            .acceleration_structures(&structures);

        let mut writes = [*WriteDescriptorSet::builder()
            .push_next(&mut acc_write)
            .dst_set(self.descriptor_sets.descriptor_sets[0])
            .dst_binding(0)
            .descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR)];

        writes[0].descriptor_count = 1;

        unsafe {
            device.vk_device().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_camera_descriptors(&mut self, device: &DeviceContext) {
        let buffer_write = [*DescriptorBufferInfo::builder()
            .range(self.camera_buffer.content_size())
            .buffer(self.camera_buffer.buffer)];

        let writes = [*WriteDescriptorSet::builder()
            .buffer_info(&buffer_write)
            .dst_set(self.descriptor_sets.descriptor_sets[1])
            .dst_binding(0)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)];

        unsafe {
            device.vk_device().update_descriptor_sets(&writes, &[]);
        }
    }
}
