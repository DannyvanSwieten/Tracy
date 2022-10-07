use std::{rc::Rc, sync::Arc};

use crate::rtx_extensions::RtxExtensions;
use nalgebra_glm::{vec4, Vec4};
use vk_utils::{device_context::DeviceContext, queue::CommandQueue};

use crate::{
    gpu_resource::{CpuResource, GpuResource},
    gpu_resource_cache::GpuResourceCache,
    image_resource::TextureImageData,
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
    pub metallic: f32,
    pub sheen: f32,
    pub clear_coat: f32,
    pub albedo_map: Option<Arc<CpuResource<TextureImageData>>>,
    pub normal_map: Option<Arc<CpuResource<TextureImageData>>>,
    pub metallic_roughness_map: Option<Arc<CpuResource<TextureImageData>>>,
    pub emission_map: Option<Arc<CpuResource<TextureImageData>>>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: vec4(0.5, 0.5, 0.5, 1.),
            emission: vec4(0., 0., 0., 0.),
            roughness: 1.0,
            metallic: 0.0,
            sheen: 0.0,
            clear_coat: 0.0,
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
    type Item = crate::material::Material;

    fn prepare(
        &self,
        _: Rc<DeviceContext>,
        _: &RtxExtensions,
        _: Rc<CommandQueue>,
        cache: &GpuResourceCache,
    ) -> Self::Item {
        let mut mat = crate::material::Material::new(
            self.base_color,
            self.emission,
            self.roughness,
            self.metallic,
            self.sheen,
            self.clear_coat,
        );

        if let Some(base_color_texture) = &self.albedo_map {
            mat = mat
                .with_base_color_texture(cache.texture(base_color_texture.uid()).unwrap().clone())
        }
        if let Some(metallic_roughness_texture) = &self.metallic_roughness_map {
            mat = mat.with_metallic_roughness_texture(
                cache
                    .texture(metallic_roughness_texture.uid())
                    .unwrap()
                    .clone(),
            )
        }
        if let Some(emission_texture) = &self.emission_map {
            mat = mat.with_emission_texture(cache.texture(emission_texture.uid()).unwrap().clone())
        }
        if let Some(normal_texture) = &self.emission_map {
            mat = mat.with_emission_texture(cache.texture(normal_texture.uid()).unwrap().clone())
        }

        mat
    }
}
