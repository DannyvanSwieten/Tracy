use std::sync::Arc;

use nalgebra_glm::{vec4, Vec4};
use renderer::context::RtxContext;

use crate::{
    image_resource::TextureImageData,
    resource::{GpuResource, Resource},
};

pub struct MaterialResource {
    pub id: usize,
    pub material: Material,
}

#[repr(C)]
#[derive(Clone)]
pub struct Material {
    pub base_color: Vec4,
    pub emission: Vec4,
    pub roughness: f32,
    pub metalness: f32,
    pub albedo_map: Option<Arc<Resource<TextureImageData>>>,
    pub normal_map: Option<Arc<Resource<TextureImageData>>>,
    pub metallic_roughness_map: Option<Arc<Resource<TextureImageData>>>,
    pub emission_map: Option<Arc<Resource<TextureImageData>>>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: vec4(0.5, 0.5, 0.5, 1.),
            roughness: 1.0,
            metalness: 0.0,
            emission: vec4(0., 0., 0., 0.),
            albedo_map: None,
            normal_map: None,
            metallic_roughness_map: None,
            emission_map: None,
        }
    }
}

impl Material {
    pub fn new(color: &Vec4) -> Self {
        Self {
            base_color: *color,
            ..Default::default()
        }
    }
}

impl GpuResource for Material {
    type Item = u32;

    fn prepare(
        &self,
        device: &vk_utils::device_context::DeviceContext,
        rtx: &RtxContext,
    ) -> Self::Item {
        todo!()
    }
}
