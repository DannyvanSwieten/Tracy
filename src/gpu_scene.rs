use std::sync::Arc;

use crate::geometry::TopLevelAccelerationStructure;
use crate::shape::Shape;

use vk_utils::buffer_resource::BufferResource;
use vk_utils::image2d_resource::Image2DResource;

use ash::vk::DescriptorSet;

pub struct GpuTexture {
    pub image: Image2DResource,
    pub image_view: ash::vk::ImageView,
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
