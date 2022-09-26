use std::rc::Rc;

use crate::context::RtxContext;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildRangeInfoKHR,
    AccelerationStructureBuildTypeKHR, AccelerationStructureCreateInfoKHR,
    AccelerationStructureDeviceAddressInfoKHR, AccelerationStructureGeometryDataKHR,
    AccelerationStructureGeometryInstancesDataKHR, AccelerationStructureGeometryKHR,
    AccelerationStructureGeometryTrianglesDataKHR, AccelerationStructureKHR,
    AccelerationStructureTypeKHR, BufferUsageFlags, BuildAccelerationStructureModeKHR,
    DeviceAddress, DeviceOrHostAddressConstKHR, DeviceOrHostAddressKHR, Format,
    GeometryInstanceFlagsKHR, GeometryTypeKHR, IndexType, MemoryPropertyFlags,
};
use ash::Device;
use glm::Mat4x3;
use vk_utils::buffer_resource::BufferResource;
use vk_utils::command_buffer::CommandBuffer;
use vk_utils::device_context::DeviceContext;
use vk_utils::queue::CommandQueue;

pub type Position = glm::Vec3;
pub type Normal = glm::Vec3;
pub type Tangent = glm::Vec3;
pub type Texcoord = glm::Vec2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GeometryInstance {
    pub transform: glm::Mat4x3,
    id_and_mask: u32,
    hit_group_offset_and_flags: u32,
    pub acceleration_structure_handle: u64,
}

impl GeometryInstance {
    pub fn new(
        instance_id: u32,
        mask: u8,
        hit_group_offset: u32,
        _flags: GeometryInstanceFlagsKHR,
        acceleration_structure_handle: u64,
        transform: Mat4x3,
    ) -> Self {
        let id_and_mask = ((mask as u32) << 24) | instance_id;
        let hit_group_offset_and_flags = ((1 as u32) << 24) | hit_group_offset;
        Self {
            transform,
            id_and_mask,
            hit_group_offset_and_flags,
            acceleration_structure_handle,
        }
    }

    pub fn geometry_id(&self) -> usize {
        self.acceleration_structure_handle as usize
    }
}

pub struct BottomLevelAccelerationStructure {
    _device: Rc<DeviceContext>,
    rtx: RtxContext,
    _acceleration_structure_buffer: BufferResource,
    _acceleration_structure_scratch_buffer: BufferResource,
    acceleration_structure: AccelerationStructureKHR,
    address: DeviceAddress,
}
impl BottomLevelAccelerationStructure {
    pub fn address(&self) -> DeviceAddress {
        self.address
    }
}
impl BottomLevelAccelerationStructure {
    pub fn new(
        device: Rc<DeviceContext>,
        rtx: &RtxContext,
        queue: Rc<CommandQueue>,
        vertex_buffer: &BufferResource,
        _vertex_count: u32,
        vertex_offset: u32,
        index_buffer: &BufferResource,
        index_count: u32,
        index_offset: u32,
    ) -> Self {
        unsafe {
            let triangles = AccelerationStructureGeometryTrianglesDataKHR::builder()
                .max_vertex(index_count - 1 + index_offset)
                .vertex_stride(12)
                .vertex_format(Format::R32G32B32_SFLOAT)
                .vertex_data(DeviceOrHostAddressConstKHR {
                    device_address: vertex_buffer.device_address(),
                })
                .index_type(IndexType::UINT32)
                .index_data(DeviceOrHostAddressConstKHR {
                    device_address: index_buffer.device_address(),
                });

            let data = AccelerationStructureGeometryDataKHR {
                triangles: *triangles,
            };

            let geometries = [AccelerationStructureGeometryKHR::builder()
                .geometry(data)
                .geometry_type(GeometryTypeKHR::TRIANGLES)
                .build()];

            let build_info = AccelerationStructureBuildGeometryInfoKHR::builder()
                .geometries(&geometries)
                .mode(BuildAccelerationStructureModeKHR::BUILD)
                .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL);

            let max_primitives: [u32; 1] = [index_count / 3];
            let build_sizes = rtx
                .acceleration_structure_ext()
                .get_acceleration_structure_build_sizes(
                    AccelerationStructureBuildTypeKHR::HOST_OR_DEVICE,
                    &build_info,
                    &max_primitives,
                );
            let scratch_buffer = BufferResource::new(
                device.clone(),
                build_sizes.build_scratch_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                    | BufferUsageFlags::STORAGE_BUFFER,
            );

            let acc_buffer = BufferResource::new(
                device.clone(),
                build_sizes.acceleration_structure_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            let create_info = AccelerationStructureCreateInfoKHR::builder()
                .size(build_sizes.acceleration_structure_size)
                .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .buffer(acc_buffer.buffer);

            let acceleration_structure = rtx
                .acceleration_structure_ext()
                .create_acceleration_structure(&create_info, None)
                .expect("Acceleration structure creation failed");
            let build_info = *AccelerationStructureBuildGeometryInfoKHR::builder()
                .dst_acceleration_structure(acceleration_structure)
                .scratch_data(DeviceOrHostAddressKHR {
                    device_address: scratch_buffer.device_address(),
                })
                .geometries(&geometries)
                .mode(BuildAccelerationStructureModeKHR::BUILD)
                .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL);

            let infos = [build_info];

            let range = vec![*AccelerationStructureBuildRangeInfoKHR::builder()
                .primitive_count(index_count / 3)
                .primitive_offset(index_offset * std::mem::size_of::<u32>() as u32)
                .first_vertex(vertex_offset)];
            let ranges = vec![&range[0..1]];

            let mut command_buffer = CommandBuffer::new(device.clone(), queue.clone());
            command_buffer.begin();
            command_buffer.record_handle(|handle| {
                rtx.acceleration_structure_ext()
                    .cmd_build_acceleration_structures(handle, &infos, &ranges);
                handle
            });
            command_buffer.submit();

            let address_info = AccelerationStructureDeviceAddressInfoKHR::builder()
                .acceleration_structure(acceleration_structure)
                .build();
            let address = rtx
                .acceleration_structure_ext()
                .get_acceleration_structure_device_address(&address_info);

            Self {
                rtx: rtx.clone(),
                _device: device.clone(),
                _acceleration_structure_buffer: acc_buffer,
                _acceleration_structure_scratch_buffer: scratch_buffer,
                acceleration_structure,
                address,
            }
        }
    }
}

impl Drop for BottomLevelAccelerationStructure {
    fn drop(&mut self) {
        unsafe {
            self.rtx
                .acceleration_structure_ext()
                .destroy_acceleration_structure(self.acceleration_structure, None)
        }
    }
}

pub struct TopLevelAccelerationStructure {
    _device: Device,
    rtx: RtxContext,
    pub acceleration_structure: AccelerationStructureKHR,
    _instance_buffer: BufferResource,
    _acceleration_structure_buffer: BufferResource,
}

impl Drop for TopLevelAccelerationStructure {
    fn drop(&mut self) {
        unsafe {
            self.rtx
                .acceleration_structure_ext()
                .destroy_acceleration_structure(self.acceleration_structure, None)
        }
    }
}

impl TopLevelAccelerationStructure {
    pub fn new(
        device: Rc<DeviceContext>,
        rtx: &RtxContext,
        queue: Rc<CommandQueue>,
        instances: &[GeometryInstance],
    ) -> Self {
        let mut _instance_buffer = BufferResource::new(
            device.clone(),
            instances.len() as u64 * 64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        _instance_buffer.upload(instances);

        let data = AccelerationStructureGeometryDataKHR {
            instances: AccelerationStructureGeometryInstancesDataKHR::builder()
                .data(DeviceOrHostAddressConstKHR {
                    device_address: _instance_buffer.device_address(),
                })
                .build(),
        };

        let geometries = [AccelerationStructureGeometryKHR::builder()
            .geometry(data)
            .geometry_type(GeometryTypeKHR::INSTANCES)
            .build()];

        let build_info = AccelerationStructureBuildGeometryInfoKHR::builder()
            .geometries(&geometries)
            .mode(BuildAccelerationStructureModeKHR::BUILD)
            .ty(AccelerationStructureTypeKHR::TOP_LEVEL);

        let max_primitives = [instances.len() as u32];

        unsafe {
            let build_sizes = rtx
                .acceleration_structure_ext()
                .get_acceleration_structure_build_sizes(
                    AccelerationStructureBuildTypeKHR::HOST_OR_DEVICE,
                    &build_info,
                    &max_primitives,
                );

            let scratch_buffer = BufferResource::new(
                device.clone(),
                build_sizes.build_scratch_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                    | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                    | BufferUsageFlags::STORAGE_BUFFER,
            );

            let acc_buffer = BufferResource::new(
                device.clone(),
                build_sizes.acceleration_structure_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                    | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            );

            let create_info = AccelerationStructureCreateInfoKHR::builder()
                .size(build_sizes.acceleration_structure_size)
                .ty(AccelerationStructureTypeKHR::TOP_LEVEL)
                .buffer(acc_buffer.buffer);

            let acceleration_structure = rtx
                .acceleration_structure_ext()
                .create_acceleration_structure(&create_info, None)
                .expect("Acceleration structure creation failed");
            let build_info = AccelerationStructureBuildGeometryInfoKHR::builder()
                .dst_acceleration_structure(acceleration_structure)
                .scratch_data(DeviceOrHostAddressKHR {
                    device_address: scratch_buffer.device_address(),
                })
                .geometries(&geometries)
                .mode(BuildAccelerationStructureModeKHR::BUILD)
                .ty(AccelerationStructureTypeKHR::TOP_LEVEL)
                .build();

            let infos = [build_info];

            let range = vec![AccelerationStructureBuildRangeInfoKHR::builder()
                .primitive_count(instances.len() as u32)
                .build()];
            let ranges = vec![&range[0..1]];
            let mut command_buffer = CommandBuffer::new(device.clone(), queue.clone());
            command_buffer.begin();
            command_buffer.record_handle(|handle| {
                rtx.acceleration_structure_ext()
                    .cmd_build_acceleration_structures(handle, &infos, &ranges);
                handle
            });
            command_buffer.submit();

            Self {
                _device: device.handle().clone(),
                rtx: rtx.clone(),
                acceleration_structure,
                _instance_buffer,
                _acceleration_structure_buffer: acc_buffer,
            }
        }
    }
}
