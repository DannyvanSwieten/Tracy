use crate::buffer_resource::BufferResource;
use crate::context::RtxContext;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildRangeInfoKHR,
    AccelerationStructureBuildTypeKHR, AccelerationStructureCreateInfoKHR,
    AccelerationStructureGeometryDataKHR, AccelerationStructureGeometryInstancesDataKHR,
    AccelerationStructureGeometryKHR, AccelerationStructureGeometryTrianglesDataKHR,
    AccelerationStructureInstanceKHR, AccelerationStructureKHR, AccelerationStructureTypeKHR,
    Buffer, BufferDeviceAddressInfo, BufferUsageFlags, BuildAccelerationStructureModeKHR,
    CommandBuffer, CommandBufferBeginInfo, DeviceOrHostAddressConstKHR, DeviceOrHostAddressKHR,
    Format, GeometryTypeKHR, IndexType, MemoryPropertyFlags, PhysicalDeviceMemoryProperties2,
    SubmitInfo,
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
        ctx: &RtxContext,
        vertex_buffer: &BufferResource,
        vertex_count: u32,
        vertex_offset: u32,
        index_buffer: &BufferResource,
        index_count: u32,
        index_offset: u32,
    ) -> Self {
        unsafe {
            let triangles = AccelerationStructureGeometryTrianglesDataKHR::builder()
                .max_vertex(vertex_count + vertex_offset)
                .vertex_stride(128)
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

            let max_primitives: [u32; 1] = [index_count];
            let build_sizes = ctx
                .acceleration_structure_ext()
                .get_acceleration_structure_build_sizes(
                    AccelerationStructureBuildTypeKHR::HOST_OR_DEVICE,
                    &build_info,
                    &max_primitives,
                );
            let scratch_buffer = BufferResource::new(
                &ctx.memory_properties().memory_properties,
                ctx.device(),
                build_sizes.build_scratch_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            let acc_buffer = BufferResource::new(
                &ctx.memory_properties().memory_properties,
                ctx.device(),
                build_sizes.acceleration_structure_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            let create_info = AccelerationStructureCreateInfoKHR::builder()
                .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .buffer(acc_buffer.buffer);

            let acceleration_structure = ctx
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
                .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .build();

            let infos = [build_info];

            let range = vec![AccelerationStructureBuildRangeInfoKHR::builder()
                .primitive_count(vertex_count)
                .first_vertex(vertex_offset)
                .build()];
            let ranges = vec![&range[0..1]];

            let command_buffer = ctx.command_buffer();

            ctx.device()
                .begin_command_buffer(command_buffer, &CommandBufferBeginInfo::builder().build())
                .expect("Start commanbuffer failed");

            ctx.acceleration_structure_ext()
                .cmd_build_acceleration_structures(command_buffer, &infos, &ranges);

            ctx.device()
                .end_command_buffer(command_buffer)
                .expect("End command buffer failed");

            ctx.submit_command_buffers(&command_buffer);

            ctx.device().device_wait_idle().expect("Wait failed");

            Self {
                device: ctx.device().clone(),
                acceleration_structure_buffer: acc_buffer,
                acceleration_structure_scratch_buffer: scratch_buffer,
                acceleration_structure,
            }
        }
    }
}

pub struct TopLevelAccelerationStructure {
    acceleration_structure: AccelerationStructureKHR,
    instance_buffer: BufferResource,
}

impl TopLevelAccelerationStructure {
    pub fn new(
        ctx: &RtxContext,
        blases: &[BottomLevelAccelerationStructure],
        instances: &[GeometryInstance],
    ) {
        let instance_buffer = BufferResource::new(
            &ctx.memory_properties().memory_properties,
            ctx.device(),
            instances.len() as u64 * 64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        let data = AccelerationStructureGeometryDataKHR {
            instances: AccelerationStructureGeometryInstancesDataKHR::builder()
                .data(DeviceOrHostAddressConstKHR {
                    device_address: instance_buffer.device_address(),
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
            let build_sizes = ctx
                .acceleration_structure_ext()
                .get_acceleration_structure_build_sizes(
                    AccelerationStructureBuildTypeKHR::HOST_OR_DEVICE,
                    &build_info,
                    &max_primitives,
                );

            let scratch_buffer = BufferResource::new(
                &ctx.memory_properties().memory_properties,
                ctx.device(),
                build_sizes.build_scratch_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            let acc_buffer = BufferResource::new(
                &ctx.memory_properties().memory_properties,
                ctx.device(),
                build_sizes.acceleration_structure_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            let create_info = AccelerationStructureCreateInfoKHR::builder()
                .ty(AccelerationStructureTypeKHR::TOP_LEVEL)
                .buffer(acc_buffer.buffer);

            let acceleration_structure = ctx
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
                .first_vertex(0)
                .build()];
            let ranges = vec![&range[0..1]];

            let command_buffer = ctx.command_buffer();

            ctx.device()
                .begin_command_buffer(command_buffer, &CommandBufferBeginInfo::builder().build())
                .expect("Start commanbuffer failed");

            ctx.acceleration_structure_ext()
                .cmd_build_acceleration_structures(command_buffer, &infos, &ranges);

            ctx.device()
                .end_command_buffer(command_buffer)
                .expect("End command buffer failed");

            ctx.submit_command_buffers(&command_buffer);

            ctx.device().device_wait_idle().expect("Wait failed");
        }
    }
}
