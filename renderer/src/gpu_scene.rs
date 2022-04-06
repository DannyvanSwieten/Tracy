use std::cell::RefCell;
use std::rc::Rc;

use crate::context::RtxContext;
use crate::geometry::{
    BottomLevelAccelerationStructure, GeometryInstance, Normal, Position, Tangent, Texcoord,
    TopLevelAccelerationStructure,
};
use crate::material::Material;
use crate::shape::Shape;

use glm::Mat4x4;
use vk_utils::buffer_resource::BufferResource;
use vk_utils::device_context::DeviceContext;
use vk_utils::image_resource::Image2DResource;

use ash::vk::{BufferUsageFlags, DescriptorSet, GeometryInstanceFlagsKHR, MemoryPropertyFlags};

pub struct GpuTexture {
    pub image: Image2DResource,
    pub image_view: ash::vk::ImageView,
}

pub struct Mesh {
    pub index_buffer: BufferResource,
    pub vertex_buffer: BufferResource,
    pub normal_buffer: BufferResource,
    pub tangent_buffer: BufferResource,
    pub tex_coord_buffer: BufferResource,
    pub blas: BottomLevelAccelerationStructure,
}

impl Mesh {
    pub fn new(
        device: &DeviceContext,
        rtx: &RtxContext,
        indices: &[u32],
        positions: &[Position],
        normals: &[Normal],
        tangents: &[Tangent],
        tex_coords: &[Texcoord],
    ) -> Self {
        let mut index_buffer = device.buffer(
            (indices.len() * std::mem::size_of::<u32>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        index_buffer.copy_to(&indices);

        let mut vertex_buffer = device.buffer(
            (positions.len() * std::mem::size_of::<Position>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        vertex_buffer.copy_to(&positions);

        let mut normal_buffer = device.buffer(
            (normals.len() * std::mem::size_of::<Normal>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        normal_buffer.copy_to(&normals);

        let mut tangent_buffer = device.buffer(
            (tangents.len() * std::mem::size_of::<Tangent>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        tangent_buffer.copy_to(&tangents);

        let mut tex_coord_buffer = device.buffer(
            (tex_coords.len() * std::mem::size_of::<Texcoord>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        tex_coord_buffer.copy_to(&tex_coords);

        let blas = BottomLevelAccelerationStructure::new(
            device,
            rtx,
            &vertex_buffer,
            positions.len() as u32,
            0,
            &index_buffer,
            indices.len() as u32,
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
pub struct MeshAddress {
    _index_address: ash::vk::DeviceAddress,
    _vertex_address: ash::vk::DeviceAddress,
    _normal_address: ash::vk::DeviceAddress,
    _tangent_address: ash::vk::DeviceAddress,
    _tex_coord_address: ash::vk::DeviceAddress,
}

impl MeshAddress {
    pub fn new(gpu_mesh: &Mesh) -> Self {
        Self {
            _index_address: gpu_mesh.index_buffer.device_address(),
            _vertex_address: gpu_mesh.vertex_buffer.device_address(),
            _normal_address: gpu_mesh.normal_buffer.device_address(),
            _tangent_address: gpu_mesh.tangent_buffer.device_address(),
            _tex_coord_address: gpu_mesh.tex_coord_buffer.device_address(),
        }
    }
}

#[derive(Default)]
pub struct GpuScene {
    pub instances: Vec<ResourceHandle<GeometryInstance>>,
    pub materials: Vec<ResourceHandle<Material>>,
    pub meshes: Vec<ResourceHandle<Mesh>>,
    pub textures: Vec<ResourceHandle<GpuTexture>>,
}

pub struct Scene {
    shapes: Vec<Rc<Shape>>,
}

impl Scene {
    pub fn attach_shape(&mut self, shape: Rc<Shape>) {
        self.shapes.push(shape);
    }
}

#[derive(Clone)]
pub struct ResourceHandle<T> {
    id: usize,
    data: Rc<T>,
}

impl GpuScene {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_mesh(&mut self, mesh: Rc<Mesh>) -> Rc<Mesh> {
        self.meshes.push(ResourceHandle { id: 0, data: mesh });
        self.meshes.last().unwrap().data.clone()
    }

    // fn add_material(&mut self, material: Material) {
    //     // if let Some(albedo_map) = &material.item().albedo_map {
    //     //     self.add_image(albedo_map.clone());
    //     // }
    //     // if let Some(normal_map) = &material.item().normal_map {
    //     //     self.add_image(normal_map.clone());
    //     // }
    //     self.materials.push(material)
    // }

    // fn add_image(&mut self, texture: GpuTexture) {
    //     self.textures.push(texture);
    // }

    // pub fn create_instance(&mut self, mesh_id: usize, transform: &Mat4x4) {
    //     self.instances.push(GeometryInstance::new(
    //         self.instances.len() as u32,
    //         0,
    //         0,
    //         GeometryInstanceFlagsKHR::empty(),
    //         mesh_id as u64,
    //         transform.remove_column(3),
    //     ));
    // }
}

pub struct Frame {
    pub material_buffer: BufferResource,
    pub material_address_buffer: BufferResource,
    pub address_buffer: BufferResource,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub acceleration_structure: TopLevelAccelerationStructure,
}
