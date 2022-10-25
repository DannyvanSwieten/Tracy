use std::sync::Arc;

use cgmath::vec4;

use crate::gpu_scene::GpuTexture;
use crate::math::Vec4;
use crate::uid_object::Handle;

pub struct Material {
    pub base_color: Vec4,
    pub emission: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub sheen: f32,
    pub clear_coat: f32,
    pub base_color_texture: Option<Arc<Handle<GpuTexture>>>,
    pub metallic_roughness_texture: Option<Arc<Handle<GpuTexture>>>,
    pub normal_texture: Option<Arc<Handle<GpuTexture>>>,
    pub emission_texture: Option<Arc<Handle<GpuTexture>>>,
}

impl Material {
    pub fn new(
        base_color: Vec4,
        emission: Vec4,
        roughness: f32,
        metallic: f32,
        sheen: f32,
        clear_coat: f32,
    ) -> Self {
        Self {
            base_color,
            emission,
            roughness,
            metallic,
            sheen,
            clear_coat,
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_texture: None,
            emission_texture: None,
        }
    }

    pub fn with_base_color_texture(mut self, texture: Arc<Handle<GpuTexture>>) -> Self {
        self.base_color_texture = Some(texture);
        self
    }

    pub fn with_normal_texture(mut self, texture: Arc<Handle<GpuTexture>>) -> Self {
        self.normal_texture = Some(texture);
        self
    }

    pub fn with_metallic_roughness_texture(mut self, texture: Arc<Handle<GpuTexture>>) -> Self {
        self.metallic_roughness_texture = Some(texture);
        self
    }

    pub fn with_emission_texture(mut self, texture: Arc<Handle<GpuTexture>>) -> Self {
        self.emission_texture = Some(texture);
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: vec4(1., 1., 1., 1.),
            emission: Vec4::new(0.0, 0.0, 0.0, 0.0),
            roughness: 1.0,
            metallic: 0.0,
            sheen: 0.0,
            clear_coat: 0.0,
            base_color_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
            emission_texture: None,
        }
    }
}
#[repr(C)]
pub struct GpuMaterial {
    pub _base_color: Vec4,
    pub _emission: Vec4,
    pub _roughness: f32,
    pub _metallic: f32,
    pub _sheen: f32,
    pub _clear_coat: f32,
    pub _base_color_texture: i32,
    pub _metallic_roughness_texture: i32,
    pub _normal_texture: i32,
    pub _emission_texture: i32,
}

impl GpuMaterial {
    pub fn new(
        material: &Material,
        base_color_texture: i32,
        metallic_roughness_texture: i32,
        normal_texture: i32,
        emission_texture: i32,
    ) -> Self {
        Self {
            _base_color: material.base_color,
            _emission: material.emission,
            _roughness: material.roughness,
            _metallic: material.metallic,
            _sheen: material.sheen,
            _clear_coat: material.clear_coat,
            _base_color_texture: base_color_texture,
            _metallic_roughness_texture: metallic_roughness_texture,
            _normal_texture: normal_texture,
            _emission_texture: emission_texture,
        }
    }
}