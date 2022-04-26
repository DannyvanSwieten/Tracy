use std::sync::Arc;

use renderer::{
    context::RtxContext,
    geometry::{Normal, Position, Tangent, Texcoord},
    gpu_scene::Mesh,
};
use vk_utils::device_context::DeviceContext;

use crate::{
    material_resource::{Material, MaterialResource},
    resource::{GpuResource, Resource},
    resources::GpuResourceCache,
};

pub struct MeshResource {
    pub indices: Vec<u32>,
    pub positions: Vec<Position>,
    pub normals: Vec<Normal>,
    pub tangents: Vec<Tangent>,
    pub tex_coords: Vec<Texcoord>,
    pub material: Arc<Resource<Material>>,
}

impl MeshResource {
    pub fn new(
        indices: Vec<u32>,
        positions: Vec<Position>,
        normals: Vec<Normal>,
        tangents: Vec<Tangent>,
        tex_coords: Vec<Texcoord>,
        material: Arc<Resource<Material>>,
    ) -> Self {
        Self {
            indices,
            positions,
            normals,
            tangents,
            tex_coords,
            material,
        }
    }

    pub fn with_material(mut self, material: Arc<Resource<Material>>) -> Self {
        self.material = material;
        self
    }
}

impl GpuResource for MeshResource {
    type Item = Mesh;

    fn prepare(
        &self,
        device: &DeviceContext,
        rtx: &RtxContext,
        _: &GpuResourceCache,
    ) -> Self::Item {
        // Turn Cpu mesh into Gpu mesh
        Mesh::new(
            device,
            rtx,
            &self.indices,
            &self.positions,
            &self.normals,
            &self.tangents,
            &self.tex_coords,
        )
    }
}