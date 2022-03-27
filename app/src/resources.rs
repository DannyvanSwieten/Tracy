use renderer::cpu_scene::{CpuMesh, TextureImageData};

use super::image_resource::TextureResource;
use super::mesh_resource::MeshResource;

#[derive(Default)]
pub struct CpuResourceCache {
    uid: usize,
    pub images: Vec<TextureResource>,
    pub meshes: Vec<MeshResource>,
}

impl CpuResourceCache {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn next_uid(&mut self) -> usize {
        self.uid += 1;
        self.uid
    }

    pub fn add_mesh(&mut self, mesh: CpuMesh) -> usize {
        let id = self.uid;
        self.meshes.push(MeshResource { id, mesh });
        self.next_uid();
        id
    }

    pub fn add_texture(&mut self, image: TextureImageData) -> usize {
        let id = self.uid;
        self.images.push(TextureResource { id, image });
        self.next_uid();
        id
    }
}
