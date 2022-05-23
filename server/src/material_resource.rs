use std::{rc::Rc, sync::Arc};

use nalgebra_glm::{vec4, Vec4};
use renderer::context::RtxContext;
use vk_utils::{device_context::DeviceContext, queue::CommandQueue};

use crate::{
    image_resource::TextureImageData,
    resource::{GpuResource, Resource},
    resources::GpuResourceCache,
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
    pub albedo_map: Option<Arc<Resource<TextureImageData>>>,
    pub normal_map: Option<Arc<Resource<TextureImageData>>>,
    pub metallic_roughness_map: Option<Arc<Resource<TextureImageData>>>,
    pub emission_map: Option<Arc<Resource<TextureImageData>>>,
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
    type Item = renderer::material::Material;

    fn prepare(
        &self,
        _: Rc<DeviceContext>,
        _: &RtxContext,
        _: Rc<CommandQueue>,
        cache: &GpuResourceCache,
    ) -> Self::Item {
        let mut mat = renderer::material::Material::new(
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
