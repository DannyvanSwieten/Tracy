use crate::context::RtxContext;
use crate::cpu_scene::{Material, Scene, TextureImageData};
use crate::geometry::{
    BottomLevelAccelerationStructure, GeometryInstance, GeometryOffset, Normal, Tangent, Texcoord,
    TopLevelAccelerationStructure, Vertex,
};
use crate::mesh_resource::MeshResource;
use glm::Mat4x4;
use std::collections::HashMap;
use vk_utils::buffer_resource::BufferResource;
use vk_utils::device_context::DeviceContext;
use vk_utils::image_resource::Image2DResource;

use ash::vk::{BufferUsageFlags, DescriptorSet, GeometryInstanceFlagsKHR, MemoryPropertyFlags};

pub struct GpuTexture {
    image: Image2DResource,
    pub image_view: ash::vk::ImageView,
}

pub struct GpuMesh {
    pub index_buffer: BufferResource,
    pub vertex_buffer: BufferResource,
    pub normal_buffer: BufferResource,
    pub tangent_buffer: BufferResource,
    pub tex_coord_buffer: BufferResource,
    pub blas: BottomLevelAccelerationStructure,
}

impl GpuMesh {
    pub fn new(device: &DeviceContext, rtx: &RtxContext, mesh: &MeshResource) -> Self {
        let mut index_buffer = device.buffer(
            (mesh.indices.len() * std::mem::size_of::<u32>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        index_buffer.copy_to(&mesh.indices);

        let mut vertex_buffer = device.buffer(
            (mesh.positions.len() * std::mem::size_of::<Vertex>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        vertex_buffer.copy_to(&mesh.positions);

        let mut normal_buffer = device.buffer(
            (mesh.normals.len() * std::mem::size_of::<Normal>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        normal_buffer.copy_to(&mesh.normals);

        let mut tangent_buffer = device.buffer(
            (mesh.tangents.len() * std::mem::size_of::<Tangent>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        tangent_buffer.copy_to(&mesh.tangents);

        let mut tex_coord_buffer = device.buffer(
            (mesh.tex_coords.len() * std::mem::size_of::<Texcoord>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        tex_coord_buffer.copy_to(&mesh.tex_coords);

        let blas = BottomLevelAccelerationStructure::new(
            device,
            rtx,
            &vertex_buffer,
            mesh.positions.len() as u32,
            0,
            &index_buffer,
            mesh.indices.len() as u32,
            0,
        );

        Self {
            index_buffer,
            vertex_buffer,
            normal_buffer,
            tangent_buffer,
            tex_coord_buffer,
            blas,
        }
    }
}
#[derive(Clone)]
pub struct GpuMeshAddress {
    _index_address: ash::vk::DeviceAddress,
    _vertex_address: ash::vk::DeviceAddress,
    _normal_address: ash::vk::DeviceAddress,
    _tangent_address: ash::vk::DeviceAddress,
    _tex_coord_address: ash::vk::DeviceAddress,
}

impl GpuMeshAddress {
    pub fn new(gpu_mesh: &GpuMesh) -> Self {
        Self {
            _index_address: gpu_mesh.index_buffer.device_address(),
            _vertex_address: gpu_mesh.vertex_buffer.device_address(),
            _normal_address: gpu_mesh.normal_buffer.device_address(),
            _tangent_address: gpu_mesh.tangent_buffer.device_address(),
            _tex_coord_address: gpu_mesh.tex_coord_buffer.device_address(),
        }
    }
}

pub struct GpuResourceCache {
    pub meshes: HashMap<usize, GpuMesh>,
    pub mesh_addresses: HashMap<usize, GpuMeshAddress>,
    pub textures: HashMap<usize, GpuTexture>,
    pub samplers: HashMap<usize, ash::vk::Sampler>,
}

impl GpuResourceCache {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            mesh_addresses: HashMap::new(),
            textures: HashMap::new(),
            samplers: HashMap::new(),
        }
    }

    pub fn add_mesh(
        &mut self,
        device: &DeviceContext,
        rtx: &RtxContext,
        mesh: &MeshResource,
        id: usize,
    ) -> usize {
        if let Some(_) = self.meshes.get(&id) {
            id
        } else {
            self.meshes.insert(id, GpuMesh::new(device, rtx, mesh));
            if let Some(gpu_mesh) = self.meshes.get(&id) {
                self.mesh_addresses
                    .insert(id, GpuMeshAddress::new(gpu_mesh));
            }
            id
        }
    }

    pub fn add_texture(&mut self, device: &DeviceContext, texture: &TextureImageData, id: usize) {
        let image = device.image_2d(
            texture.width,
            texture.height,
            texture.format,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
        );

        let buffer = device.buffer(
            texture.pixels.len() as u64,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
            ash::vk::BufferUsageFlags::TRANSFER_SRC,
        );

        device
            .graphics_queue()
            .unwrap()
            .begin(|command_buffer_handle| {
                command_buffer_handle.copy_buffer_to_image_2d(&buffer, &image);
                command_buffer_handle
            });

        let view_info = *ash::vk::ImageViewCreateInfo::builder()
            .format(texture.format)
            .image(*image.vk_image())
            .subresource_range(
                *ash::vk::ImageSubresourceRange::builder()
                    .layer_count(1)
                    .level_count(1)
                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR),
            );

        let image_view = unsafe {
            device
                .vk_device()
                .create_image_view(&view_info, None)
                .expect("Image view creation failed")
        };

        self.textures.insert(id, GpuTexture { image_view, image });
    }

    pub fn buffer_addresses(&self, id: usize) -> &GpuMeshAddress {
        &self.mesh_addresses.get(&id).unwrap()
    }
}

#[derive(Default)]
pub struct GpuScene {
    pub instances: Vec<GeometryInstance>,
    pub materials: Vec<Material>,
    pub meshes: Vec<usize>,
    pub images: Vec<usize>,
    pub image_views: Vec<usize>,
    pub samplers: Vec<usize>,
}

impl GpuScene {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_mesh(&mut self, id: usize) {
        self.meshes.push(id);
    }

    pub fn add_image(&mut self, id: usize) {
        self.images.push(id);
    }

    pub fn create_instance(&mut self, mesh_id: usize, transform: &Mat4x4, material: Material) {
        self.materials.push(material);
        self.instances.push(GeometryInstance::new(
            self.instances.len() as u32,
            0,
            0,
            GeometryInstanceFlagsKHR::empty(),
            mesh_id as u64,
            transform.remove_column(3),
        ));
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }

    pub fn materials_mut(&mut self) -> &mut [Material] {
        &mut self.materials
    }
}

unsafe impl Send for GpuResourceCache {}

pub struct Frame {
    pub material_buffer: BufferResource,
    pub material_address_buffer: BufferResource,
    pub address_buffer: BufferResource,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub acceleration_structure: TopLevelAccelerationStructure,
}

pub struct SceneData {
    pub vertex_buffer: BufferResource,
    pub normal_buffer: BufferResource,
    pub index_buffer: BufferResource,
    pub tex_coord_buffer: BufferResource,
    pub offset_buffer: BufferResource,
    pub material_buffer: BufferResource,
    pub address_buffer: BufferResource,
    pub bottom_level_acceleration_structures: Vec<BottomLevelAccelerationStructure>,
    pub top_level_acceleration_structure: TopLevelAccelerationStructure,
    pub images: Vec<Image2DResource>,
    pub image_views: Vec<ash::vk::ImageView>,
    pub samplers: Vec<ash::vk::Sampler>,
}

impl SceneData {
    pub fn new(device: &DeviceContext, rtx: &RtxContext, scene: &Scene) -> Self {
        let geometry = scene.geometry_buffer();
        let mut vertex_buffer = device.buffer(
            (geometry.vertices().len() * std::mem::size_of::<Vertex>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        vertex_buffer.copy_to(geometry.vertices());

        let mut index_buffer = device.buffer(
            (geometry.indices().len() * std::mem::size_of::<u32>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        let mut normal_buffer = device.buffer(
            (geometry.normals().len() * std::mem::size_of::<glm::Vec3>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS | BufferUsageFlags::STORAGE_BUFFER,
        );

        normal_buffer.copy_to(geometry.normals());

        index_buffer.copy_to(geometry.indices());

        let mut tex_coord_buffer = device.buffer(
            (geometry.vertices().len() * std::mem::size_of::<nalgebra_glm::Vec2>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS | BufferUsageFlags::STORAGE_BUFFER,
        );

        tex_coord_buffer.copy_to(geometry.tex_coords());

        let mut offset_buffer = device.buffer(
            (scene.geometry_offsets().len() * std::mem::size_of::<GeometryOffset>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        offset_buffer.copy_to(scene.geometry_offsets());

        let mut material_buffer = device.buffer(
            (scene.materials().len() * std::mem::size_of::<Material>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        material_buffer.copy_to(scene.materials());

        let mut address_buffer = device.buffer(
            48 as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::UNIFORM_BUFFER,
        );

        address_buffer.copy_to(&[
            vertex_buffer.device_address(),
            normal_buffer.device_address(),
            index_buffer.device_address(),
            tex_coord_buffer.device_address(),
            offset_buffer.device_address(),
            material_buffer.device_address(),
        ]);

        let bottom_level_acceleration_structures: Vec<BottomLevelAccelerationStructure> = scene
            .geometry_buffer_views()
            .iter()
            .map(|view| {
                BottomLevelAccelerationStructure::new(
                    &device,
                    &rtx,
                    &vertex_buffer,
                    view.vertex_count(),
                    view.vertex_offset(),
                    &index_buffer,
                    view.index_count(),
                    view.index_offset(),
                )
            })
            .collect();

        let instances: Vec<GeometryInstance> = scene
            .geometry_instances()
            .iter()
            .enumerate()
            .map(|(i, instance)| {
                let mut ni = GeometryInstance::new(
                    i as u32,
                    0xff,
                    0,
                    GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE,
                    bottom_level_acceleration_structures[instance.geometry_id()].address(),
                    instance.transform,
                );
                ni.transform = instance.transform;
                ni
            })
            .collect();

        let top_level_acceleration_structure =
            TopLevelAccelerationStructure::new(&device, &rtx, &instances);

        let images: Vec<Image2DResource> = scene
            .images()
            .iter()
            .map(|data| {
                let mut buffer = device.buffer(
                    data.pixels.len() as _,
                    ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
                    ash::vk::BufferUsageFlags::TRANSFER_SRC,
                );

                buffer.copy_to(&data.pixels);

                let mut image = device.image_2d(
                    data.width,
                    data.height,
                    data.format,
                    ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    ash::vk::ImageUsageFlags::SAMPLED | ash::vk::ImageUsageFlags::TRANSFER_DST,
                );

                device.graphics_queue().unwrap().begin(|command_buffer| {
                    command_buffer.color_image_resource_transition(
                        &mut image,
                        ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    );

                    command_buffer.copy_buffer_to_image_2d(&buffer, &image);
                    command_buffer.color_image_resource_transition(
                        &mut image,
                        ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    );
                    command_buffer
                });

                image
            })
            .collect();

        let image_views: Vec<ash::vk::ImageView> = images
            .iter()
            .map(|image| {
                let range = ash::vk::ImageSubresourceRange::builder()
                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1);
                let info_builder = ash::vk::ImageViewCreateInfo::builder()
                    .image(*image.vk_image())
                    .view_type(ash::vk::ImageViewType::TYPE_2D)
                    .format(image.format())
                    .subresource_range(*range);
                unsafe {
                    device
                        .vk_device()
                        .create_image_view(&info_builder, None)
                        .expect("Image View Creation Failed")
                }
            })
            .collect();

        let mut samplers = Vec::new();
        unsafe {
            samplers.push(
                device
                    .vk_device()
                    .create_sampler(
                        &ash::vk::SamplerCreateInfo::builder()
                            .min_filter(ash::vk::Filter::LINEAR)
                            .mag_filter(ash::vk::Filter::LINEAR)
                            .anisotropy_enable(true)
                            .max_anisotropy(8.0),
                        None,
                    )
                    .expect("Sampler creation failed"),
            );
        }

        Self {
            vertex_buffer,
            normal_buffer,
            index_buffer,
            tex_coord_buffer,
            offset_buffer,
            material_buffer,
            address_buffer,
            bottom_level_acceleration_structures,
            top_level_acceleration_structure,
            images: images,
            image_views,
            samplers,
        }
    }
}
