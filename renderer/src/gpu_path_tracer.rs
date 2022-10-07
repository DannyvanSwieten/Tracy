use std::rc::Rc;

use crate::context::RtxExtensions;
use crate::descriptor_sets::{
    RTXDescriptorSets, ACCELERATION_STRUCTURE_LOCATION, ACCUMULATION_IMAGE_LOCATION,
    CAMERA_BUFFER_LOCATION, MATERIAL_BUFFER_ADDRESS_LOCATION, MATERIAL_TEXTURE_LOCATION,
    MESH_BUFFERS_LOCATION, OUTPUT_IMAGE_LOCATION,
};
use crate::geometry::TopLevelAccelerationStructure;
use crate::gpu_scene::{Frame, Scene};
use crate::material::GpuMaterial;
use crate::mesh::MeshAddress;
use ash::extensions::khr::{
    AccelerationStructure, DeferredHostOperations, RayTracingPipeline, Swapchain,
};
use nalgebra_glm::{vec3, Vec3};

use vk_utils::buffer_resource::BufferResource;
use vk_utils::command_buffer::CommandBuffer;
use vk_utils::device_context::DeviceContext;
use vk_utils::gpu::Gpu;
use vk_utils::image2d_resource::Image2DResource;
use vk_utils::image_resource::ImageResource;
use vk_utils::queue::CommandQueue;
use vk_utils::shader_library::load_spirv;
use vk_utils::wait_handle::WaitHandle;

// Extension Objects
use ash::vk::{
    DeferredOperationKHR, KhrPortabilitySubsetFn, PhysicalDeviceFeatures2KHR, QueueFlags,
    RayTracingPipelineCreateInfoKHR, RayTracingShaderGroupCreateInfoKHR,
    RayTracingShaderGroupTypeKHR, StridedDeviceAddressRegionKHR,
    WriteDescriptorSetAccelerationStructureKHR, SHADER_UNUSED_KHR,
};
// Core objects
use ash::vk::{
    BufferUsageFlags, DescriptorBufferInfo, DescriptorImageInfo, DescriptorType, Format,
    ImageAspectFlags, ImageLayout, ImageSubresourceRange, ImageUsageFlags, ImageView,
    ImageViewCreateInfo, ImageViewType, MemoryPropertyFlags, Pipeline, PipelineBindPoint,
    PipelineCache, PipelineShaderStageCreateInfo, ShaderModuleCreateInfo, ShaderStageFlags,
    WriteDescriptorSet,
};

struct CameraData {
    view_inverse: glm::Mat4,
    projection_inverse: glm::Mat4,
}

pub struct Renderer {
    pub device: Rc<DeviceContext>,
    pub rtx: RtxExtensions,
    pipeline: Pipeline,
    queue: Rc<CommandQueue>,
    accumulation_image: Rc<Image2DResource>,
    accumulation_image_view: ImageView,
    output_image: Rc<Image2DResource>,
    output_image_view: ImageView,
    shader_binding_table: Option<BufferResource>,
    stride_addresses: Vec<StridedDeviceAddressRegionKHR>,

    descriptor_sets: RTXDescriptorSets,
    sampler: ash::vk::Sampler,

    pub output_width: u32,
    pub output_height: u32,

    wait_handles: [Option<WaitHandle>; 3],
    current_frame_index: usize,

    camera_position: Vec3,
    camera_target: Vec3,
    current_batch: u32,
}

unsafe impl Send for Renderer {}

impl Renderer {
    pub fn create_suitable_device_windows(gpu: &Gpu) -> DeviceContext {
        let extensions = [
            RayTracingPipeline::name(),
            AccelerationStructure::name(),
            DeferredHostOperations::name(),
            Swapchain::name(),
        ];

        let mut rt_features = ash::vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::builder()
            .ray_tracing_pipeline(true);
        let mut address_features =
            ash::vk::PhysicalDeviceVulkan12Features::builder().buffer_device_address(true);
        let mut acc_features = ash::vk::PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
            .acceleration_structure(true);
        let mut features2 = ash::vk::PhysicalDeviceFeatures2KHR::default();
        unsafe {
            gpu.vulkan()
                .vk_instance()
                .get_physical_device_features2(*gpu.vk_physical_device(), &mut features2);
        }

        gpu.device_context(&extensions, |builder| {
            builder
                .push_next(&mut address_features)
                .push_next(&mut rt_features)
                .push_next(&mut acc_features)
                .enabled_features(&features2.features)
        })
    }

    pub fn create_suitable_device_mac(gpu: &Gpu) -> DeviceContext {
        let extensions = [KhrPortabilitySubsetFn::name(), Swapchain::name()];

        let mut address_features =
            ash::vk::PhysicalDeviceVulkan12Features::builder().buffer_device_address(true);
        let mut features2 = PhysicalDeviceFeatures2KHR::default();
        unsafe {
            gpu.vulkan()
                .vk_instance()
                .get_physical_device_features2(*gpu.vk_physical_device(), &mut features2);
        }

        gpu.device_context(&extensions, |builder| {
            builder
                .push_next(&mut address_features)
                .enabled_features(&features2.features)
        })
    }

    pub fn download_image(&mut self) -> BufferResource {
        self.device.wait();
        let buffer = BufferResource::new(
            self.device.clone(),
            (self.output_width * self.output_height * 4) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::TRANSFER_DST,
        );

        let mut command_buffer = CommandBuffer::new(self.device.clone(), self.queue.clone());
        command_buffer.begin();
        command_buffer.image_resource_transition(
            Rc::get_mut(&mut self.output_image).unwrap(),
            ImageLayout::TRANSFER_SRC_OPTIMAL,
        );
        command_buffer.copy_image_to_buffer(self.output_image.as_ref(), &buffer);
        command_buffer.submit();

        buffer
    }
    pub fn render_frame(&mut self, frame: &Frame, spp: u32) -> (Rc<Image2DResource>, ImageView) {
        let mut command_buffer = CommandBuffer::new(self.device.clone(), self.queue.clone());
        command_buffer.begin();
        command_buffer.image_resource_transition(
            Rc::get_mut(&mut self.accumulation_image).unwrap(),
            ImageLayout::GENERAL,
        );
        command_buffer.image_resource_transition(
            Rc::get_mut(&mut self.output_image).unwrap(),
            ImageLayout::GENERAL,
        );

        command_buffer.image_resource_transition(
            Rc::get_mut(&mut self.output_image).unwrap(),
            ImageLayout::GENERAL,
        );

        command_buffer.bind_descriptor_sets(
            &self.descriptor_sets.pipeline_layout,
            PipelineBindPoint::RAY_TRACING_KHR,
            &frame.descriptor_sets,
        );

        unsafe {
            command_buffer.record_handle(|handle| {
                self.device.handle().cmd_bind_pipeline(
                    handle,
                    PipelineBindPoint::RAY_TRACING_KHR,
                    self.pipeline,
                );
                let constants: Vec<u8> = [spp, self.current_batch]
                    .iter()
                    .flat_map(|val| {
                        let i: u32 = *val;
                        let bytes = i.to_le_bytes();
                        bytes
                    })
                    .collect();
                self.device.handle().cmd_push_constants(
                    handle,
                    self.descriptor_sets.pipeline_layout,
                    ShaderStageFlags::RAYGEN_KHR,
                    0,
                    &constants,
                );
                self.rtx.pipeline_ext().cmd_trace_rays(
                    handle,
                    &self.stride_addresses[0],
                    &self.stride_addresses[1],
                    &self.stride_addresses[2],
                    &self.stride_addresses[3],
                    self.output_width,
                    self.output_height,
                    1,
                );
                handle
            });

            self.wait_handles[self.current_frame_index] = Some(command_buffer.submit());

            self.current_frame_index += 1;
            self.current_frame_index %= 3;
            self.current_batch += 1;
        }

        (self.output_image.clone(), self.output_image_view)
    }

    pub fn queue(&self) -> Rc<CommandQueue> {
        self.queue.clone()
    }

    pub fn output_image(&self) -> Rc<Image2DResource> {
        self.output_image.clone()
    }

    pub fn build_frame(&self, scene: &Scene) -> Frame {
        let mut mesh_addresses = Vec::new();
        let mut materials = Vec::new();
        let mut image_writes = Vec::new();
        let mut instances = Vec::new();
        for shape in scene.shapes() {
            for instance in shape.instances() {
                mesh_addresses.push(MeshAddress::new(shape.mesh()));
                let material = instance.material();
                let base_color_id = if let Some(base_color) = &material.base_color_texture {
                    image_writes.push(
                        *DescriptorImageInfo::builder()
                            .image_view(base_color.image_view)
                            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .sampler(self.sampler),
                    );

                    (image_writes.len() - 1) as i32
                } else {
                    -1
                };

                let metallic_roughness_id =
                    if let Some(metallic_roughness) = &material.metallic_roughness_texture {
                        image_writes.push(
                            *DescriptorImageInfo::builder()
                                .image_view(metallic_roughness.image_view)
                                .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                                .sampler(self.sampler),
                        );

                        (image_writes.len() - 1) as i32
                    } else {
                        -1
                    };

                let normal_id = if let Some(normal) = &material.normal_texture {
                    image_writes.push(
                        *DescriptorImageInfo::builder()
                            .image_view(normal.image_view)
                            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .sampler(self.sampler),
                    );

                    (image_writes.len() - 1) as i32
                } else {
                    -1
                };

                let emission_id = if let Some(emission) = &material.emission_texture {
                    image_writes.push(
                        *DescriptorImageInfo::builder()
                            .image_view(emission.image_view)
                            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .sampler(self.sampler),
                    );

                    (image_writes.len() - 1) as i32
                } else {
                    -1
                };

                materials.push(GpuMaterial::new(
                    material,
                    base_color_id,
                    metallic_roughness_id,
                    normal_id,
                    emission_id,
                ));

                let instance_id = instances.len() as u32;
                instances.push(instance.gpu_instance(instance_id));
            }
        }

        let mut material_buffer = BufferResource::new(
            self.device.clone(),
            (materials.len() * std::mem::size_of::<GpuMaterial>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        material_buffer.upload(&materials);

        let mut mesh_address_buffer = BufferResource::new(
            self.device.clone(),
            (mesh_addresses.len() * std::mem::size_of::<MeshAddress>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        mesh_address_buffer.upload(&mesh_addresses);

        let mut material_address_buffer = BufferResource::new(
            self.device.clone(),
            (std::mem::size_of::<u64>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::UNIFORM_BUFFER,
        );

        material_address_buffer.upload(&[material_buffer.device_address()]);

        let acceleration_structure = TopLevelAccelerationStructure::new(
            self.device.clone(),
            &self.rtx,
            self.queue.clone(),
            &instances,
        );

        let descriptor_sets = self.descriptor_sets.descriptor_sets(&self.device);
        if image_writes.len() > 0 {
            let writes = [*WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[MATERIAL_TEXTURE_LOCATION.0 as usize])
                .dst_binding(MATERIAL_TEXTURE_LOCATION.1)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_writes)];
            unsafe {
                self.device.handle().update_descriptor_sets(&writes, &[]);
            }
        }

        Self::update_acceleration_structure_descriptors(
            &self.device,
            &acceleration_structure,
            &descriptor_sets,
        );

        let camera_buffer = self.build_camera_buffer(self.device.clone());
        Self::update_material_descriptor(&self.device, &descriptor_sets, &material_address_buffer);
        Self::update_mesh_address_descriptor(&self.device, &descriptor_sets, &mesh_address_buffer);
        Self::update_camera_descriptors(&self.device, &camera_buffer, &descriptor_sets);
        self.update_image_descriptors(&self.device, &descriptor_sets);

        Frame {
            material_buffer,
            material_address_buffer,
            mesh_address_buffer,
            descriptor_sets,
            acceleration_structure,
            camera_buffer,
        }
    }
}

impl Renderer {
    pub fn new(device: Rc<DeviceContext>, width: u32, height: u32) -> Self {
        let rtx = RtxExtensions::new(&device);
        let queue = Rc::new(CommandQueue::new(device.clone(), QueueFlags::GRAPHICS));
        let descriptor_sets = RTXDescriptorSets::new(&device);
        let sampler = unsafe {
            device
                .handle()
                .create_sampler(
                    &ash::vk::SamplerCreateInfo::builder()
                        .min_filter(ash::vk::Filter::LINEAR)
                        .mag_filter(ash::vk::Filter::LINEAR)
                        .anisotropy_enable(true)
                        .max_anisotropy(8.0),
                    None,
                )
                .expect("Sampler creation failed")
        };

        let d = device.clone();

        let accumulation_image = Image2DResource::new(
            device.clone(),
            width,
            height,
            Format::R32G32B32A32_SFLOAT,
            ImageUsageFlags::STORAGE,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let output_image = Image2DResource::new(
            device.clone(),
            width,
            height,
            Format::R8G8B8A8_UNORM,
            ImageUsageFlags::TRANSFER_SRC | ImageUsageFlags::STORAGE,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let mut result = Self {
            device,
            rtx,
            queue,
            pipeline: Pipeline::null(),
            accumulation_image: Rc::new(accumulation_image),
            accumulation_image_view: ImageView::null(),
            output_image: Rc::new(output_image),
            output_image_view: ImageView::null(),
            shader_binding_table: None,
            stride_addresses: Vec::new(),

            descriptor_sets,
            sampler,

            output_width: 0,
            output_height: 0,

            wait_handles: [None, None, None],
            current_frame_index: 0,

            camera_position: Vec3::new(1., 1., 5.),
            camera_target: vec3(0.0, 0.0, 0.0),
            current_batch: 0,
        };

        result.load_shaders_and_pipeline(d.clone());
        result.create_shader_binding_table(d.clone());
        result.create_images_and_views(d.clone(), width, height);
        result.output_width = width;
        result.output_height = height;
        result
    }

    fn build_camera_buffer(&self, device: Rc<DeviceContext>) -> BufferResource {
        let view_matrix = glm::look_at_rh(
            &self.camera_position,
            &self.camera_target,
            &glm::vec3(0., 1., 0.),
        );
        let view_inverse = glm::inverse(&view_matrix);

        let projection_matrix = glm::perspective_rh(
            self.output_width as f32 / self.output_height as f32,
            1.134,
            0.1,
            1000.,
        );

        let projection_inverse = glm::inverse(&projection_matrix);
        let cam_data = CameraData {
            view_inverse,
            projection_inverse,
        };
        let mut camera_buffer = BufferResource::new(
            device,
            (std::mem::size_of::<CameraData>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::UNIFORM_BUFFER,
        );
        camera_buffer.upload(&[cam_data]);
        camera_buffer
    }

    pub fn camera_position(&self) -> &Vec3 {
        &self.camera_position
    }

    fn load_shaders_and_pipeline(&mut self, device: Rc<DeviceContext>) {
        let dir = std::env::current_exe()
            .expect("current dir check failed")
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("shaders")
            .join("simple_pipeline");
        unsafe {
            let code = load_spirv(dir.join("ray_gen.rgen.spv").to_str().unwrap());
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let gen = device
                .handle()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray generation shader compilation failed");
            let code = load_spirv(dir.join("closest_hit.rchit.spv").to_str().unwrap());
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let chit = device
                .handle()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray closest hit shader compilation failed");
            let code = load_spirv(dir.join("ray_miss.rmiss.spv").to_str().unwrap());
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let miss = device
                .handle()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray miss shader compilation failed");

            let shader_groups = vec![
                // group0 = [ raygen ]
                *RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(RayTracingShaderGroupTypeKHR::GENERAL)
                    .general_shader(0)
                    .closest_hit_shader(SHADER_UNUSED_KHR)
                    .any_hit_shader(SHADER_UNUSED_KHR)
                    .intersection_shader(SHADER_UNUSED_KHR),
                // group1 = [ chit ]
                *RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP)
                    .general_shader(SHADER_UNUSED_KHR)
                    .closest_hit_shader(1)
                    .any_hit_shader(SHADER_UNUSED_KHR)
                    .intersection_shader(SHADER_UNUSED_KHR),
                // group2 = [ miss ]
                *RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(RayTracingShaderGroupTypeKHR::GENERAL)
                    .general_shader(2)
                    .closest_hit_shader(SHADER_UNUSED_KHR)
                    .any_hit_shader(SHADER_UNUSED_KHR)
                    .intersection_shader(SHADER_UNUSED_KHR),
            ];

            let shader_stages = vec![
                *PipelineShaderStageCreateInfo::builder()
                    .stage(ShaderStageFlags::RAYGEN_KHR)
                    .module(gen)
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap()),
                *PipelineShaderStageCreateInfo::builder()
                    .stage(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .module(chit)
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap()),
                *PipelineShaderStageCreateInfo::builder()
                    .stage(ShaderStageFlags::MISS_KHR)
                    .module(miss)
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap()),
            ];

            let infos = [*RayTracingPipelineCreateInfoKHR::builder()
                .stages(&shader_stages)
                .groups(&shader_groups)
                .max_pipeline_ray_recursion_depth(2)
                .layout(self.descriptor_sets.pipeline_layout)];

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

    fn create_shader_binding_table(&mut self, device: Rc<DeviceContext>) {
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

            self.shader_binding_table = Some(BufferResource::new(
                device,
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

    pub fn clear(&mut self) {
        self.current_frame_index = 0;
        self.current_batch = 0;
    }

    fn create_images_and_views(&mut self, device: Rc<DeviceContext>, width: u32, height: u32) {
        self.output_width = width;
        self.output_height = height;

        self.output_image = Rc::new(Image2DResource::new(
            self.device.clone(),
            width,
            height,
            Format::R8G8B8A8_UNORM,
            ImageUsageFlags::STORAGE | ImageUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::DEVICE_LOCAL,
        ));

        self.accumulation_image = Rc::new(Image2DResource::new(
            self.device.clone(),
            width,
            height,
            Format::R32G32B32A32_SFLOAT,
            ImageUsageFlags::STORAGE,
            MemoryPropertyFlags::DEVICE_LOCAL,
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
            .image(self.output_image.handle());

        unsafe {
            self.output_image_view = device
                .handle()
                .create_image_view(&view_info, None)
                .expect("Image View creation failed");
        }

        let accumulation_view_info = ImageViewCreateInfo::builder()
            .format(Format::R32G32B32A32_SFLOAT)
            .subresource_range(
                ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1)
                    .build(),
            )
            .view_type(ImageViewType::TYPE_2D)
            .image(self.accumulation_image.handle());

        unsafe {
            self.accumulation_image_view = device
                .handle()
                .create_image_view(&accumulation_view_info, None)
                .expect("Image View creation failed");
        }
    }

    fn update_image_descriptors(
        &self,
        device: &DeviceContext,
        descriptor_sets: &[ash::vk::DescriptorSet],
    ) {
        let image_writes = [*DescriptorImageInfo::builder()
            .image_view(self.output_image_view)
            .image_layout(ImageLayout::GENERAL)];
        let accumulation_image_writes = [*DescriptorImageInfo::builder()
            .image_view(self.accumulation_image_view)
            .image_layout(ImageLayout::GENERAL)];

        let writes = [
            *WriteDescriptorSet::builder()
                .image_info(&image_writes)
                .dst_set(descriptor_sets[OUTPUT_IMAGE_LOCATION.0 as usize])
                .dst_binding(OUTPUT_IMAGE_LOCATION.1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE),
            *WriteDescriptorSet::builder()
                .image_info(&accumulation_image_writes)
                .dst_set(descriptor_sets[ACCUMULATION_IMAGE_LOCATION.0 as usize])
                .dst_binding(ACCUMULATION_IMAGE_LOCATION.1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE),
        ];

        unsafe {
            device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_acceleration_structure_descriptors(
        device: &DeviceContext,
        top_level_acceleration_structure: &TopLevelAccelerationStructure,
        descriptor_sets: &[ash::vk::DescriptorSet],
    ) {
        let structures = [top_level_acceleration_structure
            .acceleration_structure
            .clone()];
        let mut acc_write = *WriteDescriptorSetAccelerationStructureKHR::builder()
            .acceleration_structures(&structures);

        let mut writes = [*WriteDescriptorSet::builder()
            .push_next(&mut acc_write)
            .dst_set(descriptor_sets[ACCELERATION_STRUCTURE_LOCATION.0 as usize])
            .dst_binding(ACCELERATION_STRUCTURE_LOCATION.1)
            .descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR)];

        writes[0].descriptor_count = 1;

        unsafe {
            device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_camera_descriptors(
        device: &DeviceContext,
        camera_buffer: &BufferResource,
        descriptor_sets: &[ash::vk::DescriptorSet],
    ) {
        let buffer_write = [*DescriptorBufferInfo::builder()
            .range(camera_buffer.content_size())
            .buffer(camera_buffer.buffer)];

        let writes = [*WriteDescriptorSet::builder()
            .buffer_info(&buffer_write)
            .dst_set(descriptor_sets[CAMERA_BUFFER_LOCATION.0 as usize])
            .dst_binding(CAMERA_BUFFER_LOCATION.1)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)];

        unsafe {
            device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_mesh_address_descriptor(
        device: &DeviceContext,
        descriptor_sets: &[ash::vk::DescriptorSet],
        mesh_address_buffer: &BufferResource,
    ) {
        let storage_buffer_write = [*DescriptorBufferInfo::builder()
            .range(mesh_address_buffer.content_size())
            .buffer(mesh_address_buffer.buffer)];

        let writes = [*WriteDescriptorSet::builder()
            .buffer_info(&storage_buffer_write)
            .dst_set(descriptor_sets[MESH_BUFFERS_LOCATION.0 as usize])
            .dst_binding(MESH_BUFFERS_LOCATION.1)
            .descriptor_type(DescriptorType::STORAGE_BUFFER)];

        unsafe {
            device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_material_descriptor(
        device: &DeviceContext,
        descriptor_sets: &[ash::vk::DescriptorSet],
        material_location_buffer: &BufferResource,
    ) {
        let uniform_buffer_write = [*DescriptorBufferInfo::builder()
            .range(material_location_buffer.content_size())
            .buffer(material_location_buffer.buffer)];

        let writes = [*WriteDescriptorSet::builder()
            .buffer_info(&uniform_buffer_write)
            .dst_set(descriptor_sets[MATERIAL_BUFFER_ADDRESS_LOCATION.0 as usize])
            .dst_binding(MATERIAL_BUFFER_ADDRESS_LOCATION.1)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)];

        unsafe {
            device.handle().update_descriptor_sets(&writes, &[]);
        }
    }

    pub fn set_camera_target(&mut self, x: f32, y: f32, z: f32) {
        self.camera_target = vec3(x, y, z)
    }

    pub fn set_camera_position(&mut self, x: f32, y: f32, z: f32) {
        self.camera_position = vec3(x, y, z)
    }

    pub fn move_camera_position(&mut self, dx: f32, dy: f32, dz: f32) {
        self.camera_position += vec3(dx, dy, dz)
    }
}
