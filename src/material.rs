use crate::math::Vec4;

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
