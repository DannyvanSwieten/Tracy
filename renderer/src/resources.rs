use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::gpu_resource::CpuResource;

use crate::image_resource::TextureImageData;
use crate::material_resource::Material;

use crate::mesh_resource::MeshResource;

static GLOBAL_CPU_RESOURCE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
pub struct Resources {
    pub meshes: HashMap<usize, Arc<CpuResource<MeshResource>>>,
    pub materials: HashMap<usize, Arc<CpuResource<Material>>>,
    pub textures: HashMap<usize, Arc<CpuResource<TextureImageData>>>,
    default_material_id: usize,
}

impl Resources {
    pub fn add_mesh(
        &mut self,
        origin: &str,
        name: &str,
        resource: MeshResource,
    ) -> Arc<CpuResource<MeshResource>> {
        let uid = GLOBAL_CPU_RESOURCE_ID.fetch_add(1, Ordering::SeqCst);
        let mesh = Arc::new(CpuResource::<MeshResource>::new(
            uid, origin, name, resource,
        ));
        self.meshes.insert(uid, mesh.clone());
        mesh
    }

    pub fn add_texture(
        &mut self,
        origin: &str,
        name: &str,
        resource: TextureImageData,
    ) -> Arc<CpuResource<TextureImageData>> {
        let uid = GLOBAL_CPU_RESOURCE_ID.fetch_add(1, Ordering::SeqCst);
        let texture = Arc::new(CpuResource::<TextureImageData>::new(
            uid, origin, name, resource,
        ));
        self.textures.insert(uid, texture.clone());
        texture
    }

    pub fn add_material(
        &mut self,
        origin: &str,
        name: &str,
        resource: Material,
    ) -> Arc<CpuResource<Material>> {
        let uid = GLOBAL_CPU_RESOURCE_ID.fetch_add(1, Ordering::SeqCst);
        let material = Arc::new(CpuResource::<Material>::new(uid, origin, name, resource));
        self.materials.insert(uid, material.clone());
        material
    }

    pub fn default_material(&self) -> Arc<CpuResource<Material>> {
        self.get_material_unchecked(self.default_material_id)
    }

    pub fn with_default_material(mut self) -> Self {
        self.set_default_material(Material::default());
        self
    }

    pub fn set_default_material(&mut self, resource: Material) {
        let mat = self.add_material("Internal", "Default", resource);
        self.default_material_id = mat.uid();
    }

    pub fn get_mesh_unchecked(&self, uid: usize) -> Arc<CpuResource<MeshResource>> {
        self.meshes.get(&uid).unwrap().clone()
    }
    pub fn get_texture_unchecked(&self, uid: usize) -> Arc<CpuResource<TextureImageData>> {
        self.textures.get(&uid).unwrap().clone()
    }
    pub fn get_material_unchecked(&self, uid: usize) -> Arc<CpuResource<Material>> {
        self.materials.get(&uid).unwrap().clone()
    }
}

unsafe impl Send for Resources {}
