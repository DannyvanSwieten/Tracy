use crate::context::RtxContext;
use crate::cpu_scene::Material;
use crate::descriptor_sets::{
    RTXDescriptorSets, ACCELERATION_STRUCTURE_LOCATION, ACCUMULATION_IMAGE_LOCATION,
    CAMERA_BUFFER_LOCATION, MATERIAL_BUFFER_ADDRESS_LOCATION, MATERIAL_TEXTURE_LOCATION,
    MESH_BUFFERS_LOCATION, OUTPUT_IMAGE_LOCATION,
};
use crate::geometry::{GeometryInstance, TopLevelAccelerationStructure};
use crate::gpu_scene::{Frame, GpuMeshAddress, GpuResourceCache, GpuScene, SceneData};
use nalgebra_glm::{vec3, Vec3};

use vk_utils::buffer_resource::BufferResource;
use vk_utils::device_context::DeviceContext;
use vk_utils::gpu::Gpu;
use vk_utils::image_resource::Image2DResource;
use vk_utils::shader_library::load_spirv;
use vk_utils::wait_handle::WaitHandle;

// Extension Objects
use ash::vk::{
    DeferredOperationKHR, RayTracingPipelineCreateInfoKHR, RayTracingShaderGroupCreateInfoKHR,
    RayTracingShaderGroupTypeKHR, StridedDeviceAddressRegionKHR,
    WriteDescriptorSetAccelerationStructureKHR, SHADER_UNUSED_KHR,
};
// Core objects
use ash::vk::{
    BufferUsageFlags, DescriptorBufferInfo, DescriptorImageInfo, DescriptorType, Format, Image,
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
    pub rtx: RtxContext,
    pipeline: Pipeline,
    accumulation_image: Option<Image2DResource>,
    accumulation_image_view: ImageView,
    output_image: Option<Image2DResource>,
    output_image_view: ImageView,
    camera_buffer: BufferResource,
    shader_binding_table: Option<BufferResource>,
    stride_addresses: Vec<StridedDeviceAddressRegionKHR>,

    descriptor_sets: RTXDescriptorSets,

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
    pub fn create_suitable_device(gpu: &Gpu) -> DeviceContext {
        let extensions = [
            ash::extensions::khr::RayTracingPipeline::name(),
            ash::extensions::khr::AccelerationStructure::name(),
            ash::extensions::khr::DeferredHostOperations::name(),
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
    pub fn render_frame(
        &mut self,
        device: &DeviceContext,
        frame: &Frame,
        spp: u32,
    ) -> (Image, ImageView) {
        unsafe {
            if let Some(queue) = device.graphics_queue() {
                self.wait_handles[self.current_frame_index] = Some(queue.begin(|command_buffer| {
                    if let Some(image) = self.output_image.as_mut() {
                        command_buffer.color_image_resource_transition(image, ImageLayout::GENERAL);
                        self.output_image.as_mut().unwrap().layout = ImageLayout::GENERAL;
                    } else {
                        panic!()
                    }

                    if let Some(image) = self.accumulation_image.as_mut() {
                        command_buffer.color_image_resource_transition(image, ImageLayout::GENERAL);
                        self.output_image.as_mut().unwrap().layout = ImageLayout::GENERAL;
                    } else {
                        panic!()
                    }
                    command_buffer.bind_descriptor_sets(
                        &self.descriptor_sets.pipeline_layout,
                        PipelineBindPoint::RAY_TRACING_KHR,
                        &frame.descriptor_sets,
                    );
                    device.vk_device().cmd_bind_pipeline(
                        *command_buffer.native_handle(),
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
                    device.vk_device().cmd_push_constants(
                        *command_buffer.native_handle(),
                        self.descriptor_sets.pipeline_layout,
                        ShaderStageFlags::RAYGEN_KHR,
                        0,
                        &constants,
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
                    command_buffer
                }));

                self.current_frame_index += 1;
                self.current_frame_index %= 3;
                self.current_batch += 1;
            }

            (
                *self.output_image.as_mut().unwrap().vk_image(),
                self.output_image_view,
            )
        }
    }

    pub fn build_frame(
        &self,
        device: &DeviceContext,
        rtx: &RtxContext,
        cache: &GpuResourceCache,
        mut scene: GpuScene,
    ) -> Frame {
        let descriptor_sets = self.descriptor_sets.descriptor_sets(device);

        let mut image_map = std::collections::HashMap::new();
        let image_writes: Vec<ash::vk::DescriptorImageInfo> = scene
            .image_views
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let texture = cache.textures.get(id).unwrap();
                let view = texture.image_view;
                image_map.insert(*id, index);
                *DescriptorImageInfo::builder()
                    .image_view(view)
                    .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                // .sampler(scene_data.samplers[0])
            })
            .collect();

        // for mat in scene.materials_mut() {
        //     for i in 0..4 {
        //         if mat.maps[i] != -1 {
        //             mat.maps[i] = (*image_map.get(&(mat.maps[i] as usize)).unwrap()) as i32;
        //         }
        //     }
        // }

        if image_writes.len() > 0 {
            let writes = [*WriteDescriptorSet::builder()
                .dst_binding(MATERIAL_TEXTURE_LOCATION.1)
                .dst_set(descriptor_sets[MATERIAL_TEXTURE_LOCATION.0 as usize])
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_writes)];
            unsafe {
                device.vk_device().update_descriptor_sets(&writes, &[]);
            }
        }

        let mut material_buffer = device.buffer(
            (scene.materials().len() * std::mem::size_of::<Material>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        material_buffer.copy_to(scene.materials());

        let addresses: Vec<GpuMeshAddress> = scene
            .instances
            .iter()
            .map(|instance| {
                cache
                    .mesh_addresses
                    .get(&instance.geometry_id())
                    .unwrap()
                    .clone()
            })
            .collect();

        let mut address_buffer = device.buffer(
            (addresses.len() * std::mem::size_of::<GpuMeshAddress>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        address_buffer.copy_to(&addresses);

        let instances: Vec<GeometryInstance> = scene
            .instances
            .iter()
            .enumerate()
            .map(|(i, instance)| {
                let mesh = cache
                    .meshes
                    .get(&(instance.geometry_id() as usize))
                    .unwrap();
                let ni = GeometryInstance::new(
                    i as u32,
                    0xff,
                    0,
                    ash::vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE,
                    mesh.blas.address(),
                    instance.transform,
                );
                ni
            })
            .collect();

        let acceleration_structure = TopLevelAccelerationStructure::new(&device, &rtx, &instances);

        Self::update_acceleration_structure_descriptors(
            device,
            &acceleration_structure,
            &descriptor_sets,
        );

        let mut material_address_buffer = device.buffer(
            (addresses.len() * std::mem::size_of::<u64>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::UNIFORM_BUFFER,
        );

        material_address_buffer.copy_to(&[material_buffer.device_address()]);

        Self::update_material_descriptor(device, &descriptor_sets, &material_address_buffer);
        Self::update_mesh_address_descriptor(device, &descriptor_sets, &address_buffer);
        self.update_camera_descriptors(device, &descriptor_sets);
        self.update_image_descriptors(device, &descriptor_sets);

        Frame {
            material_buffer,
            material_address_buffer,
            address_buffer,
            descriptor_sets,
            acceleration_structure,
        }
    }
}

impl Renderer {
    pub fn new(device: &DeviceContext, width: u32, height: u32) -> Self {
        let rtx = RtxContext::new(device);

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
            accumulation_image_view: ImageView::null(),
            output_image: None,
            output_image_view: ImageView::null(),
            camera_buffer,
            shader_binding_table: None,
            stride_addresses: Vec::new(),

            descriptor_sets,

            output_width: 0,
            output_height: 0,

            wait_handles: [None, None, None],
            current_frame_index: 0,

            camera_position: Vec3::new(0., 5., 12.5),
            camera_target: vec3(0.0, 0.0, 0.0),
            current_batch: 0,
        };

        result.load_shaders_and_pipeline(device);
        result.create_shader_binding_table(device);
        result.create_images_and_views(device, width, height);
        result.build_camera_buffer();
        result.output_width = width;
        result.output_height = height;
        result
    }

    fn build_camera_buffer(&mut self) {
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
        self.camera_buffer.copy_to(&[cam_data]);
    }

    pub fn move_camera(&mut self, delta: &Vec3) {
        self.camera_position += delta;
        self.clear();
        self.build_camera_buffer()
    }

    pub fn look_at(&mut self, target: &Vec3) {
        self.camera_target = *target;
        self.clear();
        self.build_camera_buffer()
    }

    pub fn set_camera_position(&mut self, position: &Vec3) {
        self.camera_position = *position;
        self.clear();
        self.build_camera_buffer()
    }

    pub fn set_camera(&mut self, origin: &Vec3, target: &Vec3) {
        self.camera_position = *origin;
        self.camera_target = *target;
        self.clear();
        self.build_camera_buffer()
    }

    pub fn camera_position(&self) -> &Vec3 {
        &self.camera_position
    }

    fn load_shaders_and_pipeline(&mut self, device: &DeviceContext) {
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
                .vk_device()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray generation shader compilation failed");
            let code = load_spirv(dir.join("closest_hit.rchit.spv").to_str().unwrap());
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let chit = device
                .vk_device()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray closest hit shader compilation failed");
            let code = load_spirv(dir.join("ray_miss.rmiss.spv").to_str().unwrap());
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let miss = device
                .vk_device()
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

    pub fn clear(&mut self) {
        self.current_frame_index = 0;
        self.current_batch = 0;
    }

    fn create_images_and_views(&mut self, device: &DeviceContext, width: u32, height: u32) {
        self.output_width = width;
        self.output_height = height;

        self.output_image = Some(device.image_2d(
            width,
            height,
            Format::R8G8B8A8_UNORM,
            MemoryPropertyFlags::DEVICE_LOCAL,
            ImageUsageFlags::STORAGE | ImageUsageFlags::TRANSFER_SRC,
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
            .image(*self.accumulation_image.as_ref().unwrap().vk_image());

        unsafe {
            self.accumulation_image_view = device
                .vk_device()
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
            device.vk_device().update_descriptor_sets(&writes, &[]);
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
            device.vk_device().update_descriptor_sets(&writes, &[]);
        }
    }

    fn update_camera_descriptors(
        &self,
        device: &DeviceContext,
        descriptor_sets: &[ash::vk::DescriptorSet],
    ) {
        let buffer_write = [*DescriptorBufferInfo::builder()
            .range(self.camera_buffer.content_size())
            .buffer(self.camera_buffer.buffer)];

        let writes = [*WriteDescriptorSet::builder()
            .buffer_info(&buffer_write)
            .dst_set(descriptor_sets[CAMERA_BUFFER_LOCATION.0 as usize])
            .dst_binding(CAMERA_BUFFER_LOCATION.1)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)];

        unsafe {
            device.vk_device().update_descriptor_sets(&writes, &[]);
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
            device.vk_device().update_descriptor_sets(&writes, &[]);
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
            device.vk_device().update_descriptor_sets(&writes, &[]);
        }
    }
}
