use std::rc::Rc;

use ash::vk::{BufferUsageFlags, MemoryPropertyFlags};
use vk_utils::{
    buffer_resource::BufferResource, device_context::DeviceContext, queue::CommandQueue,
};

use crate::{
    context::RtxExtensions,
    geometry::{BottomLevelAccelerationStructure, Normal, Position, Tangent, Texcoord},
};

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
        device: Rc<DeviceContext>,
        rtx: &RtxExtensions,
        queue: Rc<CommandQueue>,
        indices: &[u32],
        positions: &[Position],
        normals: &[Normal],
        tangents: &[Tangent],
        tex_coords: &[Texcoord],
    ) -> Self {
        let mut index_buffer = BufferResource::new(
            device.clone(),
            (indices.len() * std::mem::size_of::<u32>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        index_buffer.upload(&indices);

        let mut vertex_buffer = BufferResource::new(
            device.clone(),
            (positions.len() * std::mem::size_of::<Position>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        vertex_buffer.upload(&positions);

        let mut normal_buffer = BufferResource::new(
            device.clone(),
            (normals.len() * std::mem::size_of::<Normal>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        normal_buffer.upload(&normals);

        let mut tangent_buffer = BufferResource::new(
            device.clone(),
            (tangents.len() * std::mem::size_of::<Tangent>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        tangent_buffer.upload(&tangents);

        let mut tex_coord_buffer = BufferResource::new(
            device.clone(),
            (tex_coords.len() * std::mem::size_of::<Texcoord>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        tex_coord_buffer.upload(&tex_coords);

        let blas = BottomLevelAccelerationStructure::new(
            device.clone(),
            rtx,
            queue.clone(),
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
