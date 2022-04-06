use std::rc::Rc;

use crate::gpu_scene::GpuTexture;

pub struct Material {
    pub base_color: glm::Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub sheen: f32,
    pub clear_coat: f32,
    pub base_color_texture: Option<Rc<GpuTexture>>,
    pub metallic_roughness_texture: Option<Rc<GpuTexture>>,
    pub normal_texture: Option<Rc<GpuTexture>>,
    pub emission_texture: Option<Rc<GpuTexture>>,
}