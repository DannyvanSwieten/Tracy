use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use renderer::context::RtxContext;
use renderer::gpu_scene::{GpuTexture, Mesh};
use vk_utils::device_context::DeviceContext;

use crate::image_resource::TextureImageData;
use crate::material_resource::Material;
use crate::mesh_resource;
use crate::resource::{GpuResource, Resource};

static GLOBAL_CPU_RESOURCE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
pub struct Resources {
    pub data: HashMap<TypeId, HashMap<usize, Arc<dyn Any>>>,
}

impl Resources {
    pub fn add<T: 'static + GpuResource>(
        &mut self,
        origin: &str,
        name: &str,
        resource: T,
    ) -> Arc<Resource<T>> {
        let type_id = TypeId::of::<T>();
        if let Some(map) = self.data.get_mut(&type_id) {
            let uid = GLOBAL_CPU_RESOURCE_ID.fetch_add(1, Ordering::SeqCst);
            let any: Arc<dyn Any> =
                Arc::new(Arc::new(Resource::<T>::new(uid, origin, name, resource)));
            map.insert(uid, any.clone());
            map.get(&uid)
                .unwrap()
                .downcast_ref::<Arc<Resource<T>>>()
                .unwrap()
                .clone()
        } else {
            self.data.insert(type_id, HashMap::new());
            self.add(origin, name, resource)
        }
    }

    pub fn get_unchecked<T: 'static + GpuResource>(&self, uid: usize) -> Arc<Resource<T>> {
        let type_id = TypeId::of::<T>();
        self.data
            .get(&type_id)
            .unwrap()
            .get(&uid)
            .unwrap()
            .downcast_ref::<Arc<Resource<T>>>()
            .unwrap()
            .clone()
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
        device: &DeviceContext,
        rtx: &RtxContext,
        mesh: &Arc<Resource<mesh_resource::MeshResource>>,
    ) -> Arc<renderer::resource::Resource<Mesh>> {
        let prepared = self.meshes.get(&mesh.uid());
        if prepared.is_none() {
            self.meshes
                .insert(mesh.uid(), mesh.prepare(device, rtx, self));
        }
        self.meshes.get(&mesh.uid()).unwrap().clone()
    }

    pub fn add_material(
        &mut self,
        device: &DeviceContext,
        rtx: &RtxContext,
        material: &Arc<Resource<Material>>,
    ) -> Arc<renderer::resource::Resource<renderer::material::Material>> {
        let prepared = self.materials.get(&material.uid());
        if prepared.is_none() {
            if let Some(base_color) = &material.albedo_map {
                self.add_texture(device, rtx, &base_color);
            }

            if let Some(metallic_roughness) = &material.metallic_roughness_map {
                self.add_texture(device, rtx, &metallic_roughness);
            }

            if let Some(normal) = &material.normal_map {
                self.add_texture(device, rtx, &normal);
            }

            if let Some(emission) = &material.emission_map {
                self.add_texture(device, rtx, &emission);
            }

            self.materials
                .insert(material.uid(), material.prepare(device, rtx, self));
        }

        self.materials.get(&material.uid()).unwrap().clone()
    }

    pub fn add_texture(
        &mut self,
        device: &DeviceContext,
        rtx: &RtxContext,
        texture: &Arc<Resource<TextureImageData>>,
    ) -> Arc<renderer::resource::Resource<GpuTexture>> {
        let prepared = self.textures.get(&texture.uid());
        if prepared.is_none() {
            self.textures
                .insert(texture.uid(), texture.prepare(device, rtx, self));
        }

        self.textures.get(&texture.uid()).unwrap().clone()
    }

    pub fn get_texture(
        &self,
        uid: usize,
    ) -> Option<&Arc<renderer::resource::Resource<GpuTexture>>> {
        self.textures.get(&uid)
    }
}

unsafe impl Send for GpuResourceCache {}
