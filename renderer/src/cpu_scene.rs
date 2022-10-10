use std::sync::Arc;

use crate::{
    geometry::*,
    gpu_scene::GpuTexture,
    resource::{CpuResource, GpuResource},
};
use glm::{vec2, vec3, vec4};

#[repr(C)]
#[derive(Clone)]
pub struct TextureImageData {
    pub format: ash::vk::Format,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl TextureImageData {
    pub fn new(format: ash::vk::Format, width: u32, height: u32, pixels: &[u8]) -> Self {
        if format == ash::vk::Format::R8G8B8_UNORM {
            let mut new_pixels = Vec::new();
            for i in (0..pixels.len()).step_by(3) {
                new_pixels.push(pixels[i]);
                new_pixels.push(pixels[i + 1]);
                new_pixels.push(pixels[i + 2]);
                new_pixels.push(255);
            }
            Self {
                format: ash::vk::Format::R8G8B8A8_UNORM,
                width,
                height,
                pixels: new_pixels,
            }
        } else {
            Self {
                format,
                width,
                height,
                pixels: pixels.to_vec(),
            }
        }
    }
}

impl GpuResource for TextureImageData {
    type Item = GpuTexture;

    fn prepare(
        &self,
        device: &vk_utils::device_context::DeviceContext,
        rtx: &crate::context::RtxExtensions,
    ) -> Self::Item {
        todo!()
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Material {
    pub base_color: glm::Vec4,
    pub emission: glm::Vec4,
    pub roughness: f32,
    pub metalness: f32,
    pub albedo_map: Option<Arc<CpuResource<TextureImageData>>>,
    pub normal_map: Option<Arc<CpuResource<TextureImageData>>>,
    pub metallic_roughness_map: Option<Arc<CpuResource<TextureImageData>>>,
    pub emission_map: Option<Arc<CpuResource<TextureImageData>>>,
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
    pub fn new(color: &glm::Vec4) -> Self {
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
        rtx: &crate::context::RtxExtensions,
    ) -> Self::Item {
        todo!()
    }
}

#[derive(Default, Clone)]
pub struct SceneGraphNode {
    pub name: String,
    pub camera: Option<usize>,
    pub mesh: Option<Vec<usize>>,
    pub children: Vec<usize>,
    pub global_transform: [[f32; 4]; 4],
    pub local_transform: [[f32; 4]; 4],
}

impl SceneGraphNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn with_camera(mut self, camera_id: usize) -> Self {
        self.camera = Some(camera_id);
        self
    }

    pub fn with_mesh(mut self, mesh_id: usize) -> Self {
        if let Some(vector) = self.mesh.as_mut() {
            vector.push(mesh_id)
        } else {
            self.mesh = Some(vec![mesh_id])
        }
        self
    }

    pub fn with_children(mut self, children: &[usize]) -> Self {
        self.children.extend(children);
        self
    }
}
