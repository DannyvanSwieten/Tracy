use renderer::{
    context::RtxContext,
    geometry::{Normal, Position, Tangent, Texcoord},
    gpu_scene::Mesh,
};
use vk_utils::device_context::DeviceContext;

use crate::resource::GpuResource;

pub struct MeshResource {
    pub indices: Vec<u32>,
    pub positions: Vec<Position>,
    pub normals: Vec<Normal>,
    pub tangents: Vec<Tangent>,
    pub tex_coords: Vec<Texcoord>,
    //pub material: Option<Arc<Resource<Material>>>,
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
        }
    }

    pub fn with_tangents(mut self, tangents: Vec<Tangent>) -> Self {
        self.tangents = tangents.to_vec();
        self
    }
}

impl GpuResource for MeshResource {
    type Item = Mesh;

    fn prepare(&self, device: &DeviceContext, rtx: &RtxContext) -> Self::Item {
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
