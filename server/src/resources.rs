use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use renderer::context::RtxContext;
use renderer::gpu_scene::{GpuTexture, Mesh};
use vk_utils::device_context::DeviceContext;
use vk_utils::queue::CommandQueue;

use crate::image_resource::TextureImageData;
use crate::material_resource::Material;
use crate::mesh_resource::{self, MeshResource};
use crate::resource::Resource;

static GLOBAL_CPU_RESOURCE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
pub struct Resources {
    pub meshes: HashMap<usize, Arc<Resource<MeshResource>>>,
    pub materials: HashMap<usize, Arc<Resource<Material>>>,
    pub textures: HashMap<usize, Arc<Resource<TextureImageData>>>,
    default_material_id: usize,
}

impl Resources {
    pub fn add_mesh(
        &mut self,
        origin: &str,
        name: &str,
        resource: MeshResource,
    ) -> Arc<Resource<MeshResource>> {
        let uid = GLOBAL_CPU_RESOURCE_ID.fetch_add(1, Ordering::SeqCst);
        let mesh = Arc::new(Resource::<MeshResource>::new(uid, origin, name, resource));
        self.meshes.insert(uid, mesh.clone());
        mesh
    }

    pub fn add_texture(
        &mut self,
        origin: &str,
        name: &str,
        resource: TextureImageData,
    ) -> Arc<Resource<TextureImageData>> {
        let uid = GLOBAL_CPU_RESOURCE_ID.fetch_add(1, Ordering::SeqCst);
        let texture = Arc::new(Resource::<TextureImageData>::new(
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
    ) -> Arc<Resource<Material>> {
        let uid = GLOBAL_CPU_RESOURCE_ID.fetch_add(1, Ordering::SeqCst);
        let material = Arc::new(Resource::<Material>::new(uid, origin, name, resource));
        self.materials.insert(uid, material.clone());
        material
    }

    pub fn default_material(&self) -> Arc<Resource<Material>> {
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

    pub fn get_mesh_unchecked(&self, uid: usize) -> Arc<Resource<MeshResource>> {
        self.meshes.get(&uid).unwrap().clone()
    }
    pub fn get_texture_unchecked(&self, uid: usize) -> Arc<Resource<TextureImageData>> {
        self.textures.get(&uid).unwrap().clone()
    }
    pub fn get_material_unchecked(&self, uid: usize) -> Arc<Resource<Material>> {
        self.materials.get(&uid).unwrap().clone()
    }
}

unsafe impl Send for Resources {}

#[derive(Default)]
pub struct GpuResourceCache {
    pub meshes: HashMap<usize, Arc<renderer::resource::Resource<Mesh>>>,
    pub textures: HashMap<usize, Arc<renderer::resource::Resource<GpuTexture>>>,
    pub samplers: HashMap<usize, Arc<renderer::resource::Resource<ash::vk::Sampler>>>,
    pub materials: HashMap<usize, Arc<renderer::resource::Resource<renderer::material::Material>>>,
}

impl GpuResourceCache {
    pub fn add_mesh(
        &mut self,
        device: Rc<DeviceContext>,
        rtx: &RtxContext,
        queue: Rc<CommandQueue>,
        mesh: &Arc<Resource<mesh_resource::MeshResource>>,
    ) -> Arc<renderer::resource::Resource<Mesh>> {
        let prepared = self.meshes.get(&mesh.uid());
        if prepared.is_none() {
            self.meshes
                .insert(mesh.uid(), mesh.prepare(device, rtx, queue, self));
        }
        self.meshes.get(&mesh.uid()).unwrap().clone()
    }

    pub fn add_material(
        &mut self,
        device: Rc<DeviceContext>,
        rtx: &RtxContext,
        queue: Rc<CommandQueue>,
        material: &Arc<Resource<Material>>,
    ) -> Arc<renderer::resource::Resource<renderer::material::Material>> {
        let prepared = self.materials.get(&material.uid());
        if prepared.is_none() {
            if let Some(base_color) = &material.albedo_map {
                self.add_texture(device.clone(), rtx, queue.clone(), &base_color);
            }

            if let Some(metallic_roughness) = &material.metallic_roughness_map {
                self.add_texture(device.clone(), rtx, queue.clone(), &metallic_roughness);
            }

            if let Some(normal) = &material.normal_map {
                self.add_texture(device.clone(), rtx, queue.clone(), &normal);
            }

            if let Some(emission) = &material.emission_map {
                self.add_texture(device.clone(), rtx, queue.clone(), &emission);
            }

            self.materials
                .insert(material.uid(), material.prepare(device, rtx, queue, self));
        }

        self.materials.get(&material.uid()).unwrap().clone()
    }

    pub fn add_texture(
        &mut self,
        device: Rc<DeviceContext>,
        rtx: &RtxContext,
        queue: Rc<CommandQueue>,
        texture: &Arc<Resource<TextureImageData>>,
    ) -> Arc<renderer::resource::Resource<GpuTexture>> {
        let prepared = self.textures.get(&texture.uid());
        if prepared.is_none() {
            self.textures
                .insert(texture.uid(), texture.prepare(device, rtx, queue, self));
        }

        self.textures.get(&texture.uid()).unwrap().clone()
    }

    pub fn texture(&self, uid: usize) -> Option<&Arc<renderer::resource::Resource<GpuTexture>>> {
        self.textures.get(&uid)
    }

    pub fn material(
        &self,
        uid: usize,
    ) -> Option<&Arc<renderer::resource::Resource<renderer::material::Material>>> {
        self.materials.get(&uid)
    }
}

unsafe impl Send for GpuResourceCache {}
