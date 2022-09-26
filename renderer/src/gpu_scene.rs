use std::rc::Rc;
use std::sync::Arc;

use crate::context::RtxContext;
use crate::geometry::{
    BottomLevelAccelerationStructure, Normal, Position, Tangent, Texcoord,
    TopLevelAccelerationStructure,
};
use crate::shape::Shape;

use vk_utils::buffer_resource::BufferResource;
use vk_utils::device_context::DeviceContext;
use vk_utils::image2d_resource::Image2DResource;

use ash::vk::{BufferUsageFlags, DescriptorSet, MemoryPropertyFlags};
use vk_utils::queue::CommandQueue;

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
        device: Rc<DeviceContext>,
        rtx: &RtxContext,
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
#[derive(Default)]
pub struct Scene {
    shapes: Vec<Arc<Shape>>,
}

impl Scene {
    pub fn attach_shape(&mut self, shape: Arc<Shape>) {
        self.shapes.push(shape);
    }

    pub fn shapes(&self) -> &Vec<Arc<Shape>> {
        &self.shapes
    }
}
pub struct Frame {
    pub material_buffer: BufferResource,
    pub material_address_buffer: BufferResource,
    pub mesh_address_buffer: BufferResource,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub camera_buffer: BufferResource,
    pub acceleration_structure: TopLevelAccelerationStructure,
}
