use ash::extensions::khr::AccelerationStructure;
use ash::extensions::khr::DeferredHostOperations;
use ash::extensions::khr::RayTracingPipeline;
use ash::vk::BufferUsageFlags;
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
use cgmath::SquareMatrix;
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
use crate::math::Mat4;
use crate::math::Vec3;
use crate::math::Vec4;
use crate::mesh::Mesh;
use crate::mesh::MeshAddress;
use crate::mesh_resource::MeshResource;
use crate::rtx_extensions::RtxExtensions;
use crate::rtx_pipeline::RtxPipeline;
use crate::scene::Scene;

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
    materials: Map<Material2>,
    default_sampler: Sampler,
}

pub struct MeshInstance {
    mesh: Handle,
    material: Handle,
    transform: Mat4,
}

impl MeshInstance {
    fn new(mesh: Handle, material: Handle) -> Self {
        Self {
            mesh,
            material,
            transform: Mat4::identity(),
        }
    }

    pub fn mesh(&self) -> Handle {
        self.mesh
    }

    pub fn set_material(&mut self, material: Handle) -> &mut Self {
        self.material = material;
        self
    }

    pub fn material(&self) -> Handle {
        self.material
    }

    pub fn transform(&self) -> &Mat4 {
        &self.transform
    }

    pub fn scale(&mut self, scale: &Vec3) -> &mut Self {
        self.transform = self.transform * Mat4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        self
    }

    pub fn translate(&mut self, translation: &Vec3) -> &mut Self {
        self.transform = self.transform * Mat4::from_translation(*translation);
        self
    }

    pub fn rotate(&mut self, rotation: &Vec3) -> &mut Self {
        self.transform = self.transform * Mat4::from_angle_x(cgmath::Deg(rotation.x));
        self.transform = self.transform * Mat4::from_angle_y(cgmath::Deg(rotation.y));
        self.transform = self.transform * Mat4::from_angle_z(cgmath::Deg(rotation.z));
        self
    }
}
pub struct Material2 {
    pub base_color: Vec4,
    pub emission: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub sheen: f32,
    pub clear_coat: f32,
    pub base_color_texture: Option<Handle>,
    pub metallic_roughness_texture: Option<Handle>,
    pub normal_texture: Option<Handle>,
    pub emission_texture: Option<Handle>,
}

impl Material2 {
    pub fn new() -> Self {
        Self {
            base_color: Vec4::new(0.5, 0.5, 0.5, 1.0),
            emission: Vec4::new(0.0, 0.0, 0.0, 0.0),
            roughness: 1.0,
            metallic: 0.0,
            sheen: 0.0,
            clear_coat: 0.0,
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_texture: None,
            emission_texture: None,
        }
    }
}

impl Default for Material2 {
    fn default() -> Self {
        Self::new()
    }
}

impl Ctx {
    pub fn new(device: Rc<DeviceContext>, max_frames_in_flight: u32) -> Self {
        let rtx = Rc::new(RtxExtensions::new(&device));
        let queue = Rc::new(CommandQueue::new(device.clone(), QueueFlags::GRAPHICS));
        let sampler_info = *SamplerCreateInfo::builder();
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

    pub fn create_framebuffer(&self, width: u32, height: u32) -> FrameBuffer {
        FrameBuffer::new(self.device.clone(), self.queue.clone(), width, height)
    }

    pub fn create_material(&mut self) -> Handle {
        self.materials.insert(Material2::new())
    }

    pub fn material_mut(&mut self, material: Handle) -> Option<&mut Material2> {
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
        let mut image = Image2DResource::new(
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

        let mut command_buffer = CommandBuffer::new(self.device.clone(), self.queue.clone());
        command_buffer.begin();
        command_buffer
            .image_resource_transition(&mut image, ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL);

        command_buffer.copy_buffer_to_image(&buffer, &mut image);

        command_buffer
            .image_resource_transition(&mut image, ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        command_buffer.submit();

        let view_info = *ash::vk::ImageViewCreateInfo::builder()
            .format(data.format)
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
                    GeometryInstanceFlagsKHR::FORCE_OPAQUE,
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
