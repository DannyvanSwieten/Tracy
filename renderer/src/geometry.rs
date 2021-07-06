use crate::context::RtxContext;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildRangeInfoKHR,
    AccelerationStructureBuildTypeKHR, AccelerationStructureCreateInfoKHR,
    AccelerationStructureDeviceAddressInfoKHR, AccelerationStructureGeometryDataKHR,
    AccelerationStructureGeometryInstancesDataKHR, AccelerationStructureGeometryKHR,
    AccelerationStructureGeometryTrianglesDataKHR, AccelerationStructureInstanceKHR,
    AccelerationStructureKHR, AccelerationStructureTypeKHR, BufferUsageFlags,
    BuildAccelerationStructureModeKHR, DeviceAddress, DeviceOrHostAddressConstKHR,
    DeviceOrHostAddressKHR, Format, GeometryInstanceFlagsKHR, GeometryTypeKHR, IndexType,
    MemoryPropertyFlags,
};
use ash::Device;
use vk_utils::buffer_resource::BufferResource;
use vk_utils::device_context::DeviceContext;

#[repr(C)]
#[derive(Default)]
pub struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

pub struct GeometryBuffer {
    indices: Vec<u32>,
    vertices: Vec<Vertex>,
}

impl GeometryBuffer {
    pub fn new() -> Self {
        Self {
            indices: Vec::default(),
            vertices: Vec::default(),
        }
    }

    pub fn append(&mut self, indices: Vec<u32>, vertices: Vec<Vertex>) {
        self.indices.extend(indices);
        self.vertices.extend(vertices);
    }

    pub fn new_with_data(indices: Vec<u32>, vertices: Vec<Vertex>) -> Self {
        Self { indices, vertices }
    }
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
}

#[repr(C)]
pub struct GeometryBufferView {
    index_count: u32,
    index_offset: u32,
    vertex_count: u32,
    vertex_offset: u32,
}

impl GeometryBufferView {
    pub fn new(index_count: u32, index_offset: u32, vertex_count: u32, vertex_offset: u32) -> Self {
        Self {
            index_count,
            index_offset,
            vertex_count,
            vertex_offset,
        }
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }
    pub fn index_offset(&self) -> u32 {
        self.index_offset
    }
    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }
    pub fn vertex_offset(&self) -> u32 {
        self.vertex_offset
    }
}

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
        flags: GeometryInstanceFlagsKHR,
        acceleration_structure_handle: u64,
    ) -> Self {
        let id_and_mask = ((mask as u32) << 24) | instance_id;
        let hit_group_offset_and_flags = ((1 as u32) << 24) | hit_group_offset;
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
    _device: Device,
    _acceleration_structure_buffer: BufferResource,
    _acceleration_structure_scratch_buffer: BufferResource,
    _acceleration_structure: AccelerationStructureKHR,
    address: DeviceAddress,
}
impl BottomLevelAccelerationStructure {
    pub fn address(&self) -> DeviceAddress {
        self.address
    }
}
impl BottomLevelAccelerationStructure {
    pub fn new(
        device: &DeviceContext,
        rtx: &RtxContext,
        vertex_buffer: &BufferResource,
        vertex_count: u32,
        vertex_offset: u32,
        index_buffer: &BufferResource,
        index_count: u32,
        index_offset: u32,
    ) -> Self {
        unsafe {
            let triangles = AccelerationStructureGeometryTrianglesDataKHR::builder()
                .max_vertex(vertex_count - 1 + vertex_offset)
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

            let max_primitives: [u32; 1] = [index_count];
            let build_sizes = rtx
                .acceleration_structure_ext()
                .get_acceleration_structure_build_sizes(
                    AccelerationStructureBuildTypeKHR::HOST_OR_DEVICE,
                    &build_info,
                    &max_primitives,
                );
            let scratch_buffer = device.buffer(
                build_sizes.build_scratch_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            let acc_buffer = device.buffer(
                build_sizes.acceleration_structure_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            let create_info = AccelerationStructureCreateInfoKHR::builder()
                .size(build_sizes.acceleration_structure_size)
                .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .buffer(acc_buffer.buffer);

            let _acceleration_structure = rtx
                .acceleration_structure_ext()
                .create_acceleration_structure(&create_info, None)
                .expect("Acceleration structure creation failed");
            let build_info = AccelerationStructureBuildGeometryInfoKHR::builder()
                .dst_acceleration_structure(_acceleration_structure)
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

            if let Some(queue) = device.graphics_queue() {
                queue.begin(|command_buffer| {
                    rtx.acceleration_structure_ext()
                        .cmd_build_acceleration_structures(
                            *command_buffer.native_handle(),
                            &infos,
                            &ranges,
                        );
                    command_buffer
                });

                device.wait();
            }

            let address_info = AccelerationStructureDeviceAddressInfoKHR::builder()
                .acceleration_structure(_acceleration_structure)
                .build();
            let address = rtx
                .acceleration_structure_ext()
                .get_acceleration_structure_device_address(&address_info);

            Self {
                _device: device.vk_device().clone(),
                _acceleration_structure_buffer: acc_buffer,
                _acceleration_structure_scratch_buffer: scratch_buffer,
                _acceleration_structure,
                address,
            }
        }
    }
}

pub struct TopLevelAccelerationStructure {
    _device: Device,
    pub acceleration_structure: AccelerationStructureKHR,
    _instance_buffer: BufferResource,
    _acceleration_structure_buffer: BufferResource,
}

impl TopLevelAccelerationStructure {
    pub fn new(device: &DeviceContext, rtx: &RtxContext, instances: &[GeometryInstance]) -> Self {
        let mut _instance_buffer = device.buffer(
            instances.len() as u64 * 64,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        _instance_buffer.copy_to(instances);

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

            let scratch_buffer = device.buffer(
                build_sizes.build_scratch_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            let acc_buffer = device.buffer(
                build_sizes.acceleration_structure_size,
                MemoryPropertyFlags::DEVICE_LOCAL,
                BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
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
                .first_vertex(0)
                .build()];
            let ranges = vec![&range[0..1]];
            if let Some(queue) = device.graphics_queue() {
                queue.begin(|command_buffer| {
                    rtx.acceleration_structure_ext()
                        .cmd_build_acceleration_structures(
                            *command_buffer.native_handle(),
                            &infos,
                            &ranges,
                        );
                    command_buffer
                });

                device.wait();
            }

            Self {
                _device: device.vk_device().clone(),
                acceleration_structure,
                _instance_buffer,
                _acceleration_structure_buffer: acc_buffer,
            }
        }
    }
}
