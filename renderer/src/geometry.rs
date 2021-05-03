use crate::buffer_resource::BufferResource;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildRangeInfoKHR,
    AccelerationStructureBuildTypeKHR, AccelerationStructureCreateInfoKHR,
    AccelerationStructureGeometryDataKHR, AccelerationStructureGeometryKHR,
    AccelerationStructureGeometryTrianglesDataKHR, AccelerationStructureKHR,
    AccelerationStructureTypeKHR, Buffer, BufferDeviceAddressInfo, BufferUsageFlags,
    BuildAccelerationStructureModeKHR, CommandBuffer, CommandBufferBeginInfo,
    DeviceOrHostAddressConstKHR, DeviceOrHostAddressKHR, Format, GeometryTypeKHR, IndexType,
    MemoryPropertyFlags, PhysicalDeviceMemoryProperties2,
};

use ash::version::{DeviceV1_0, DeviceV1_2};

use ash::extensions::khr::AccelerationStructure;
use ash::Device;

#[repr(C)]
pub struct GeometryInstance {
    transform: [f32; 12],
    id_and_mask: u32,
    hit_group_offset_and_flags: u32,
    acceleration_structure_handle: u64,
}

impl GeometryInstance {
    pub fn new(
        instance_id: u32,
        mask: u8,
        hit_group_offset: u32,
        flags: u8,
        acceleration_structure_handle: u64,
    ) -> Self {
        let id_and_mask = (instance_id << 8) | mask as u32;
        let hit_group_offset_and_flags = (hit_group_offset << 8) | flags as u32;
        let transform: [f32; 12] = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0.];
        Self {
            transform,
            id_and_mask,
            hit_group_offset_and_flags,
            acceleration_structure_handle,
        }
    }
}

pub struct BottomLevelAccelerationStructure {
    device: Device,
    acceleration_structure_buffer: BufferResource,
    acceleration_structure_scratch_buffer: BufferResource,
    acceleration_structure: AccelerationStructureKHR,
}

impl BottomLevelAccelerationStructure {
    pub fn new(
        acceleration_structure_extension: &AccelerationStructure,
        device: &Device,
        command_buffer: CommandBuffer,
        memory_properties: &PhysicalDeviceMemoryProperties2,
        vertex_buffer: &Buffer,
        index_bufer: &Buffer,
        vertex_count: u32,
        vertex_offset: u32,
    ) -> Self {
        unsafe {
            let v_address_info = BufferDeviceAddressInfo::builder().buffer(*vertex_buffer);
            let v_address = device.get_buffer_device_address(&v_address_info);
            let i_address_info = BufferDeviceAddressInfo::builder().buffer(*index_bufer);
            let i_address = device.get_buffer_device_address(&i_address_info);
            let triangles = AccelerationStructureGeometryTrianglesDataKHR::builder()
                .max_vertex(vertex_count + vertex_offset)
                .vertex_stride(128)
                .vertex_format(Format::R32G32B32_SFLOAT)
                .vertex_data(DeviceOrHostAddressConstKHR {
                    device_address: v_address,
                })
                .index_type(IndexType::UINT32)
                .index_data(DeviceOrHostAddressConstKHR {
                    device_address: i_address,
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

            let max_primitives: [u32; 1] = [10];
            let build_sizes = acceleration_structure_extension
                .get_acceleration_structure_build_sizes(
                    AccelerationStructureBuildTypeKHR::HOST_OR_DEVICE,
                    &build_info,
                    &max_primitives,
                );
            let scratch_buffer = BufferResource::new(
                &memory_properties.memory_properties,
                device,
                build_sizes.build_scratch_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            let scratch_buffer_address = device.get_buffer_device_address(
                &BufferDeviceAddressInfo::builder()
                    .buffer(scratch_buffer.buffer)
                    .build(),
            );

            let acc_buffer = BufferResource::new(
                &memory_properties.memory_properties,
                device,
                build_sizes.acceleration_structure_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            let create_info = AccelerationStructureCreateInfoKHR::builder()
                .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .buffer(scratch_buffer.buffer);

            let acceleration_structure = acceleration_structure_extension
                .create_acceleration_structure(&create_info, None)
                .expect("Acceleration structure creation failed");
            let build_info = AccelerationStructureBuildGeometryInfoKHR::builder()
                .dst_acceleration_structure(acceleration_structure)
                .scratch_data(DeviceOrHostAddressKHR {
                    device_address: scratch_buffer_address,
                })
                .geometries(&geometries)
                .mode(BuildAccelerationStructureModeKHR::BUILD)
                .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .build();

            let infos = [build_info];

            let range = vec![AccelerationStructureBuildRangeInfoKHR::builder()
                .primitive_count(vertex_count)
                .first_vertex(vertex_offset)
                .build()];
            let ranges = vec![&range[0..1]];

            device
                .begin_command_buffer(command_buffer, &CommandBufferBeginInfo::builder().build())
                .expect("Start commanbuffer failed");

            acceleration_structure_extension.cmd_build_acceleration_structures(
                command_buffer,
                &infos,
                &ranges,
            );

            device
                .end_command_buffer(command_buffer)
                .expect("End command buffer failed");

            device.device_wait_idle().expect("Wait failed");

            Self {
                device: device.clone(),
                acceleration_structure_buffer: acc_buffer,
                acceleration_structure_scratch_buffer: scratch_buffer,
                acceleration_structure,
            }
        }
    }
}
