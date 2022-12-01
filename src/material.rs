use crate::{ctx::Handle, math::Vec4};

pub struct Material {
    pub base_color: Vec4,
    pub emission: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub sheen: f32,
    pub clear_coat: f32,
    pub ior: f32,
    pub transmission: f32,
    pub base_color_texture: Option<Handle>,
    pub metallic_roughness_texture: Option<Handle>,
    pub normal_texture: Option<Handle>,
    pub emission_texture: Option<Handle>,
}

impl Material {
    pub fn new() -> Self {
        Self {
            base_color: Vec4::new(0.5, 0.5, 0.5, 1.0),
            emission: Vec4::new(0.0, 0.0, 0.0, 0.0),
            roughness: 1.0,
            metallic: 0.0,
            sheen: 0.0,
            clear_coat: 0.0,
            ior: 1.0,
            transmission: 0.0,
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_texture: None,
            emission_texture: None,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
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
    pub _ior: f32,
    pub _transmission: f32,
    pub _base_color_texture: i32,
    pub _metallic_roughness_texture: i32,
    pub _normal_texture: i32,
    pub _emission_texture: i32,
}
