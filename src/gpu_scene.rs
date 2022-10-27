use crate::geometry::TopLevelAccelerationStructure;

use vk_utils::buffer_resource::BufferResource;
use vk_utils::image2d_resource::Image2DResource;

use ash::vk::DescriptorSet;

pub struct GpuTexture {
    pub image: Image2DResource,
    pub image_view: ash::vk::ImageView,
}

pub struct Frame {
    pub material_buffer: BufferResource,
    pub material_address_buffer: BufferResource,
    pub mesh_address_buffer: BufferResource,
    pub descriptor_sets: Vec<DescriptorSet>,
    pub camera_buffer: BufferResource,
    pub acceleration_structure: TopLevelAccelerationStructure,
}
