use std::sync::Arc;

use vk_utils::device_context::DeviceContext;

use crate::{
    context::RtxContext,
    cpu_scene::Material,
    geometry::{Normal, Position, Tangent, Texcoord},
    gpu_scene::GpuMesh,
    resource::{GpuResource, Resource},
};

pub struct MeshResource {
    pub indices: Vec<u32>,
    pub positions: Vec<Position>,
    pub normals: Vec<Normal>,
    pub tangents: Vec<Tangent>,
    pub tex_coords: Vec<Texcoord>,
    pub material: Option<Arc<Resource<Material>>>,
}

impl MeshResource {
    pub fn new(indices: Vec<u32>, positions: Vec<Position>, normals: Vec<Normal>) -> Self {
        let tangents = positions
            .iter()
            .map(|_| Tangent::new(0.0, 0.0, 0.0))
            .collect();

        let tex_coords = positions.iter().map(|_| Texcoord::new(0.0, 0.0)).collect();

        Self {
            indices,
            positions,
            normals,
            tangents,
            tex_coords,
            material: None,
        }
    }

    pub fn with_tangents(mut self, tangents: Vec<Tangent>) -> Self {
        self.tangents = tangents.to_vec();
        self
    }
}

impl GpuResource for MeshResource {
    type Item = GpuMesh;

    fn prepare(&self, device: &DeviceContext, rtx: &RtxContext) -> Self::Item {
        // Turn Cpu mesh into Gpu mesh
        GpuMesh::new(device, rtx, &self)
    }
}
