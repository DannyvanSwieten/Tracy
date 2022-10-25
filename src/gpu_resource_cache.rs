use std::{collections::HashMap, rc::Rc, sync::Arc};

use vk_utils::{device_context::DeviceContext, queue::CommandQueue};

use crate::{
    gpu_resource::CpuResource, gpu_scene::GpuTexture, image_resource::TextureImageData,
    material::Material, mesh::Mesh, mesh_resource::MeshResource, rtx_extensions::RtxExtensions,
    uid_object::Handle,
};

#[derive(Default)]
pub struct GpuResourceCache {
    pub meshes: HashMap<usize, Arc<Handle<Mesh>>>,
    pub textures: HashMap<usize, Arc<Handle<GpuTexture>>>,
    pub samplers: HashMap<usize, Arc<Handle<ash::vk::Sampler>>>,
    pub materials: HashMap<usize, Arc<Handle<Material>>>,
}

impl GpuResourceCache {
    pub fn add_mesh(
        &mut self,
        device: Rc<DeviceContext>,
        rtx: &RtxExtensions,
        queue: Rc<CommandQueue>,
        mesh: &Arc<CpuResource<MeshResource>>,
    ) -> Arc<Handle<Mesh>> {
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
        rtx: &RtxExtensions,
        queue: Rc<CommandQueue>,
        material: &Arc<CpuResource<crate::material_resource::Material>>,
    ) -> Arc<Handle<Material>> {
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
        rtx: &RtxExtensions,
        queue: Rc<CommandQueue>,
        texture: &Arc<CpuResource<TextureImageData>>,
    ) -> Arc<Handle<GpuTexture>> {
        let prepared = self.textures.get(&texture.uid());
        if prepared.is_none() {
            self.textures
                .insert(texture.uid(), texture.prepare(device, rtx, queue, self));
        }

        self.textures.get(&texture.uid()).unwrap().clone()
    }

    pub fn texture(&self, uid: usize) -> Option<&Arc<Handle<GpuTexture>>> {
        self.textures.get(&uid)
    }

    pub fn material(&self, uid: usize) -> Option<&Arc<Handle<crate::material::Material>>> {
        self.materials.get(&uid)
    }
}

unsafe impl Send for GpuResourceCache {}