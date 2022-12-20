use ash::extensions::khr::AccelerationStructure;
use ash::extensions::khr::DeferredHostOperations;
use ash::extensions::khr::RayTracingPipeline;
use ash::vk::BufferUsageFlags;
use ash::vk::Filter;
use ash::vk::Format;
use ash::vk::GeometryInstanceFlagsKHR;
use ash::vk::ImageLayout;
use ash::vk::ImageView;
use ash::vk::KhrPortabilitySubsetFn;
use ash::vk::MemoryPropertyFlags;
use ash::vk::PhysicalDeviceFeatures2KHR;
use ash::vk::PipelineBindPoint;
use ash::vk::QueueFlags;
use ash::vk::Sampler;
use ash::vk::SamplerCreateInfo;
use ash::vk::ShaderStageFlags;
use slotmap::DefaultKey;
use slotmap::SlotMap;
use std::collections::HashMap;
use std::rc::Rc;
use vk_utils::buffer_resource::BufferResource;
use vk_utils::command_buffer::CommandBuffer;
use vk_utils::device_context::DeviceContext;
use vk_utils::gpu::Gpu;
use vk_utils::image2d_resource::Image2DResource;
use vk_utils::image_resource::ImageResource;
use vk_utils::queue::CommandQueue;

use crate::camera::Camera;
use crate::descriptor_sets::FrameDescriptors;
use crate::framebuffer::FrameBuffer;
use crate::geometry::GeometryInstance;
use crate::geometry::TopLevelAccelerationStructure;
use crate::gpu_scene::GpuTexture;
use crate::image_resource::TextureImageData;
use crate::material::GpuMaterial;
use crate::material::Material;
use crate::math::Mat4;
use crate::mesh::Mesh;
use crate::mesh::MeshAddress;
use crate::mesh_instance::MeshInstance;
use crate::mesh_resource::MeshResource;
use crate::rtx_extensions::RtxExtensions;
use crate::rtx_pipeline::RtxPipeline;
use crate::scene::Scene;
use crate::skybox::SkyBox;

pub type Handle = DefaultKey;
type Map<V> = SlotMap<Handle, V>;

pub struct FrameResources {
    pub cpu_resources: CpuResources,
    pub gpu_resources: GpuResources,
    descriptors: Rc<FrameDescriptors>,
}

pub struct CpuResources {
    pub gpu_materials: Vec<GpuMaterial>,
    pub gpu_instances: Vec<GeometryInstance>,
    pub instance_properties: Vec<InstanceProperties>,
    pub geometry_addresses: Vec<MeshAddress>,
    pub camera: Camera,
    pub skybox: Option<SkyBox>,
}

impl CpuResources {
    fn material_size(&self) -> u64 {
        std::mem::size_of::<GpuMaterial>() as u64 * self.gpu_materials.len() as u64
    }

    fn instance_property_size(&self) -> u64 {
        std::mem::size_of::<InstanceProperties>() as u64 * self.instance_properties.len() as u64
    }

    fn geometry_addresses_size(&self) -> u64 {
        std::mem::size_of::<MeshAddress>() as u64 * self.geometry_addresses.len() as u64
    }
}
pub struct GpuResources {
    pub image_views: Vec<ImageView>,
    pub sampler: Sampler,
    pub acceleration_structure: TopLevelAccelerationStructure,
    pub instance_property_buffer: BufferResource,
    pub material_buffer: BufferResource,
    pub geometry_address_buffer: BufferResource,
    pub buffer_address_buffer: BufferResource,
    pub camera_buffer: BufferResource,
    pub output_image_views: [ImageView; 2],
    pub skybox_image_view: ImageView,
}

pub struct InstanceProperties {
    geometry_index: u32,
    material_index: u32,
}

pub struct Ctx {
    device: Rc<DeviceContext>,
    rtx: Rc<RtxExtensions>,
    queue: Rc<CommandQueue>,
    pipeline: RtxPipeline,
    textures: Map<GpuTexture>,
    meshes: Map<Mesh>,
    instances: Map<MeshInstance>,
    default_material: Handle,
    materials: Map<Material>,
    default_sampler: Sampler,
    default_skybox: SkyBox,
}

impl Ctx {
    pub fn new(device: Rc<DeviceContext>, max_frames_in_flight: u32) -> Self {
        let rtx = Rc::new(RtxExtensions::new(&device));
        let queue = Rc::new(CommandQueue::new(device.clone(), QueueFlags::GRAPHICS));
        let sampler_info = *SamplerCreateInfo::builder()
            .mag_filter(Filter::LINEAR)
            .min_filter(Filter::LINEAR);
        let default_sampler = unsafe {
            device
                .handle()
                .create_sampler(&sampler_info, None)
                .expect("Sampler creation failed")
        };
        let mut instance = Self {
            device: device.clone(),
            rtx: rtx.clone(),
            pipeline: RtxPipeline::new(device, rtx, max_frames_in_flight),
            textures: Map::new(),
            meshes: Map::new(),
            instances: Map::new(),
            default_material: Handle::default(),
            materials: Map::new(),
            queue,
            default_sampler,
            default_skybox: SkyBox {
                gpu_texture_handle: Handle::default(),
            },
        };

        let skybox_image =
            TextureImageData::new(Format::R8G8B8A8_UNORM, 1, 1, &[228, 246, 248, 255]);
        let skybox_image_handle = instance.create_texture(&skybox_image);
        instance.default_skybox = SkyBox {
            gpu_texture_handle: skybox_image_handle,
        };
        let default_material = instance.create_material();
        instance.default_material = default_material;
        instance
    }

    pub fn create_suitable_device_windows(gpu: &Gpu) -> DeviceContext {
        let extensions = [
            RayTracingPipeline::name(),
            AccelerationStructure::name(),
            DeferredHostOperations::name(),
        ];

        let mut rt_features = ash::vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::builder()
            .ray_tracing_pipeline(true);
        let mut address_features = ash::vk::PhysicalDeviceVulkan12Features::builder()
            .buffer_device_address(true)
            .shader_input_attachment_array_dynamic_indexing(true)
            .descriptor_indexing(true)
            .runtime_descriptor_array(true);
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
        let extensions = [KhrPortabilitySubsetFn::name()];

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

    pub fn create_skybox(&mut self, data: &TextureImageData) -> SkyBox {
        let gpu_texture_handle = self.create_texture(data);
        SkyBox { gpu_texture_handle }
    }

    pub fn create_framebuffer(&self, width: u32, height: u32) -> FrameBuffer {
        FrameBuffer::new(self.device.clone(), self.queue.clone(), width, height)
    }

    pub fn create_material(&mut self) -> Handle {
        self.materials.insert(Material::new())
    }

    pub fn material_mut(&mut self, material: Handle) -> Option<&mut Material> {
        self.materials.get_mut(material)
    }

    pub fn create_mesh(&mut self, mesh: &MeshResource) -> Handle {
        let m = Mesh::new(
            self.device.clone(),
            &self.rtx,
            self.queue.clone(),
            &mesh.indices,
            &mesh.vertices,
            &mesh.normals,
            &mesh.tangents,
            &mesh.tex_coords,
        );

        self.meshes.insert(m)
    }

    pub fn create_texture(&mut self, data: &TextureImageData) -> Handle {
        let (mut image, buffer, format) = if data.format == Format::R8G8B8_UINT {
            let mut pixels = Vec::new();

            for i in (0..data.pixels.len()).step_by(3) {
                pixels.push(data.pixels[i]);
                pixels.push(data.pixels[i + 1]);
                pixels.push(data.pixels[i + 2]);
                pixels.push(255);
            }

            let data =
                TextureImageData::new(Format::R8G8B8A8_UNORM, data.width, data.height, &pixels);

            let image = Image2DResource::new(
                self.device.clone(),
                data.width,
                data.height,
                data.format,
                ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
                ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            );

            let mut buffer = BufferResource::new(
                self.device.clone(),
                pixels.len() as u64,
                ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
                ash::vk::BufferUsageFlags::TRANSFER_SRC,
            );

            buffer.upload(&pixels);
            (image, buffer, data.format)
        } else if data.format == Format::R8G8B8A8_UINT {
            let image = Image2DResource::new(
                self.device.clone(),
                data.width,
                data.height,
                Format::R8G8B8A8_UNORM,
                ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
                ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            );

            let mut buffer = BufferResource::new(
                self.device.clone(),
                data.pixels.len() as u64,
                ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
                ash::vk::BufferUsageFlags::TRANSFER_SRC,
            );

            buffer.upload(&data.pixels);

            (image, buffer, Format::R8G8B8A8_UNORM)
        } else {
            let image = Image2DResource::new(
                self.device.clone(),
                data.width,
                data.height,
                data.format,
                ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
                ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            );

            let mut buffer = BufferResource::new(
                self.device.clone(),
                data.pixels.len() as u64,
                ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
                ash::vk::BufferUsageFlags::TRANSFER_SRC,
            );

            buffer.upload(&data.pixels);

            (image, buffer, data.format)
        };

        let mut command_buffer = CommandBuffer::new(self.device.clone(), self.queue.clone());
        command_buffer.begin();
        command_buffer
            .image_resource_transition(&mut image, ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL);

        command_buffer.copy_buffer_to_image(&buffer, &mut image);

        command_buffer
            .image_resource_transition(&mut image, ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        command_buffer.submit();

        let view_info = *ash::vk::ImageViewCreateInfo::builder()
            .format(format)
            .view_type(ash::vk::ImageViewType::TYPE_2D)
            .image(image.handle())
            .subresource_range(
                *ash::vk::ImageSubresourceRange::builder()
                    .layer_count(1)
                    .level_count(1)
                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR),
            );

        let image_view = unsafe {
            self.device
                .handle()
                .create_image_view(&view_info, None)
                .expect("Image view creation failed")
        };

        self.textures.insert(GpuTexture { image, image_view })
    }

    pub fn upload_frame(
        &self,
        image_views: Vec<ImageView>,
        sampler: &Sampler,
        framebuffer: &FrameBuffer,
        frame: &CpuResources,
    ) -> GpuResources {
        let mut material_buffer = BufferResource::new(
            self.device.clone(),
            frame.material_size(),
            MemoryPropertyFlags::DEVICE_LOCAL | MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        material_buffer.upload(&frame.gpu_materials);

        let material_address = material_buffer.device_address();

        let mut instance_property_buffer = BufferResource::new(
            self.device.clone(),
            frame.instance_property_size(),
            MemoryPropertyFlags::DEVICE_LOCAL | MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        instance_property_buffer.upload(&frame.instance_properties);
        let instance_property_buffer_address = instance_property_buffer.device_address();

        let mut buffer_address_buffer = BufferResource::new(
            self.device.clone(),
            std::mem::size_of::<u64>() as u64 * 2,
            MemoryPropertyFlags::DEVICE_LOCAL | MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::UNIFORM_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        buffer_address_buffer.upload(&[material_address, instance_property_buffer_address]);

        let mut geometry_address_buffer = BufferResource::new(
            self.device.clone(),
            frame.geometry_addresses_size(),
            MemoryPropertyFlags::DEVICE_LOCAL | MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER,
        );

        geometry_address_buffer.upload(&frame.geometry_addresses);

        let acceleration_structure = TopLevelAccelerationStructure::new(
            self.device.clone(),
            &self.rtx,
            self.queue.clone(),
            &frame.gpu_instances,
        );

        let camera_matrices = [
            frame.camera.view_matrix(),
            frame.camera.projection_matrix(framebuffer.aspect_ratio()),
        ];

        let camera_size = std::mem::size_of::<Mat4>() * camera_matrices.len();
        let mut camera_buffer = BufferResource::new(
            self.device.clone(),
            camera_size as u64,
            MemoryPropertyFlags::DEVICE_LOCAL | MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::UNIFORM_BUFFER,
        );

        camera_buffer.upload(&camera_matrices);

        let output_image_views = [
            framebuffer.final_image_view,
            framebuffer.accumulation_image_view,
        ];

        let skybox_image_view = if let Some(skybox) = &frame.skybox {
            self.textures[skybox.gpu_texture_handle].image_view
        } else {
            self.textures[self.default_skybox.gpu_texture_handle].image_view
        };

        GpuResources {
            acceleration_structure,
            material_buffer,
            geometry_address_buffer,
            instance_property_buffer,
            image_views,
            buffer_address_buffer,
            camera_buffer,
            output_image_views,
            sampler: *sampler,
            skybox_image_view,
        }
    }

    pub fn create_instance(&mut self, mesh: Handle) -> Handle {
        self.instances
            .insert(MeshInstance::new(mesh, self.default_material))
    }

    pub fn instance_mut(&mut self, handle: Handle) -> Option<&mut MeshInstance> {
        self.instances.get_mut(handle)
    }

    pub fn build_frame_resources(
        &mut self,
        framebuffer: &FrameBuffer,
        scene: &Scene,
    ) -> FrameResources {
        let mut geometry_map = HashMap::new();
        let mut geometries = Vec::new();
        let mut geometry_addresses = Vec::new();
        for (index, (key, geometry)) in self.meshes.iter().enumerate() {
            geometry_map.insert(key, index);
            geometries.push(geometry);
            geometry_addresses.push(MeshAddress::new(geometry));
        }

        let mut texture_map = HashMap::new();
        let mut textures = Vec::new();
        let mut material_image_views = Vec::new();
        for (index, (key, texture)) in self.textures.iter().enumerate() {
            texture_map.insert(key, index);
            textures.push(texture);
            material_image_views.push(texture.image_view);
        }

        let mut material_map = HashMap::new();
        let mut materials = Vec::new();
        let mut gpu_materials = Vec::new();
        for (index, (key, material)) in self.materials.iter().enumerate() {
            material_map.insert(key, index);
            materials.push(material);
            let base_color_id = if let Some(texture_id) = material.base_color_texture {
                *texture_map.get(&texture_id).unwrap() as i32
            } else {
                -1
            };

            let emission_id = if let Some(texture_id) = material.emission_texture {
                *texture_map.get(&texture_id).unwrap() as i32
            } else {
                -1
            };

            let metal_roughness_id = if let Some(texture_id) = material.metallic_roughness_texture {
                *texture_map.get(&texture_id).unwrap() as i32
            } else {
                -1
            };

            let normal_id = if let Some(texture_id) = material.normal_texture {
                *texture_map.get(&texture_id).unwrap() as i32
            } else {
                -1
            };

            gpu_materials.push(GpuMaterial {
                _base_color: material.base_color,
                _emission: material.emission,
                _roughness: material.roughness,
                _metallic: material.metallic,
                _clear_coat: material.clear_coat,
                _sheen: material.sheen,
                _ior: material.ior,
                _transmission: material.transmission,
                _base_color_texture: base_color_id,
                _emission_texture: emission_id,
                _metallic_roughness_texture: metal_roughness_id,
                _normal_texture: normal_id,
            });
        }

        let mut gpu_instances = Vec::new();
        let mut instance_properties = Vec::new();
        for (instance_id, key) in scene.instances().iter().enumerate() {
            if let Some(instance) = self.instances.get(*key) {
                let geometry_index = *geometry_map.get(&instance.mesh()).unwrap();
                let mesh = &geometries[geometry_index];
                gpu_instances.push(GeometryInstance::new(
                    instance_id as u32,
                    0xff,
                    0,
                    GeometryInstanceFlagsKHR::FORCE_OPAQUE
                        | GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE,
                    mesh.blas.address(),
                    instance.transform(),
                ));

                instance_properties.push(InstanceProperties {
                    geometry_index: geometry_index as u32,
                    material_index: *material_map.get(&instance.material()).unwrap() as u32,
                });
            }
        }

        let cpu_resources = CpuResources {
            gpu_materials,
            gpu_instances,
            geometry_addresses,
            instance_properties,
            camera: *scene.camera(),
            skybox: scene.skybox(),
        };

        let gpu_resources = self.upload_frame(
            material_image_views,
            &self.default_sampler,
            framebuffer,
            &cpu_resources,
        );
        let descriptors = self.pipeline.descriptor_sets.next(&gpu_resources);

        FrameResources {
            cpu_resources,
            gpu_resources,
            descriptors,
        }
    }

    pub fn render_frame(
        &self,
        framebuffer: &mut FrameBuffer,
        frame: &FrameResources,
        pass_count: u32,
        samples_per_pass: u32,
    ) {
        for pass in 0..pass_count {
            let mut command_buffer = CommandBuffer::new(self.device.clone(), self.queue.clone());
            command_buffer.begin();
            if framebuffer.final_image.layout() != ImageLayout::GENERAL {
                command_buffer
                    .image_resource_transition(&mut framebuffer.final_image, ImageLayout::GENERAL);
            }
            if framebuffer.accumulation_image.layout() != ImageLayout::GENERAL {
                command_buffer.image_resource_transition(
                    &mut framebuffer.accumulation_image,
                    ImageLayout::GENERAL,
                );
            }
            command_buffer.bind_descriptor_sets(
                &self.pipeline.descriptor_sets.pipeline_layout,
                PipelineBindPoint::RAY_TRACING_KHR,
                &frame.descriptors.sets,
            );
            command_buffer
                .bind_pipeline(PipelineBindPoint::RAY_TRACING_KHR, &self.pipeline.pipeline);
            unsafe {
                command_buffer.record_handle(|handle| {
                    self.device.handle().cmd_bind_pipeline(
                        handle,
                        PipelineBindPoint::RAY_TRACING_KHR,
                        self.pipeline.pipeline,
                    );
                    let constants: Vec<u8> = [samples_per_pass, pass]
                        .iter()
                        .flat_map(|val| {
                            let i: u32 = *val;
                            i.to_le_bytes()
                        })
                        .collect();
                    self.device.handle().cmd_push_constants(
                        handle,
                        self.pipeline.descriptor_sets.pipeline_layout,
                        ShaderStageFlags::RAYGEN_KHR,
                        0,
                        &constants,
                    );
                    self.rtx.pipeline_ext().cmd_trace_rays(
                        handle,
                        &self.pipeline.stride_addresses[0],
                        &self.pipeline.stride_addresses[1],
                        &self.pipeline.stride_addresses[2],
                        &self.pipeline.stride_addresses[3],
                        framebuffer.width,
                        framebuffer.height,
                        1,
                    );
                    handle
                });
            }

            command_buffer.submit();
        }
    }
}
