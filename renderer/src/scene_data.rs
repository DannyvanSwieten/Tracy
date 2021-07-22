use crate::context::RtxContext;
use crate::geometry::{
    BottomLevelAccelerationStructure, GeometryBufferView, GeometryInstance,
    TopLevelAccelerationStructure,
};
use crate::scene::{Material, Scene};
use vk_utils::buffer_resource::BufferResource;
use vk_utils::device_context::DeviceContext;

use ash::vk::{BufferUsageFlags, GeometryInstanceFlagsKHR, MemoryPropertyFlags};

pub struct SceneData {
    pub vertex_buffer: BufferResource,
    pub index_buffer: BufferResource,
    pub offset_buffer: BufferResource,
    pub material_buffer: BufferResource,
    pub address_buffer: BufferResource,
    pub bottom_level_acceleration_structures: Vec<BottomLevelAccelerationStructure>,
    pub top_level_acceleration_structure: TopLevelAccelerationStructure,
}

impl SceneData {
    pub fn new(device: &DeviceContext, rtx: &RtxContext, scene: &Scene) -> Self {
        let geometry = scene.geometry_buffer();
        let mut vertex_buffer = device.buffer(
            (geometry.vertices().len() * 4 * 3) as u64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::VERTEX_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        vertex_buffer.copy_to(geometry.vertices());

        let mut index_buffer = device.buffer(
            (geometry.indices().len() * 4) as u64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::INDEX_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        index_buffer.copy_to(geometry.indices());

        let mut offset_buffer = device.buffer(
            (scene.geometry_buffer_views().len() * std::mem::size_of::<GeometryBufferView>())
                as u64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        offset_buffer.copy_to(scene.geometry_buffer_views());

        let mut material_buffer = device.buffer(
            (scene.materials().len() * std::mem::size_of::<Material>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        material_buffer.copy_to(scene.materials());

        let mut address_buffer = device.buffer(
            32 as u64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::UNIFORM_BUFFER,
        );

        address_buffer.copy_to(&[
            vertex_buffer.device_address(),
            index_buffer.device_address(),
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
                GeometryInstance::new(
                    i as u32,
                    0xff,
                    0,
                    GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE,
                    bottom_level_acceleration_structures[instance.geometry_id()].address(),
                )
            })
            .collect();

        let top_level_acceleration_structure =
            TopLevelAccelerationStructure::new(&device, &rtx, &instances);

        Self {
            vertex_buffer,
            index_buffer,
            offset_buffer,
            material_buffer,
            address_buffer,
            bottom_level_acceleration_structures,
            top_level_acceleration_structure,
        }
    }
}
