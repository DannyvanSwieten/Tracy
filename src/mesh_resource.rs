use std::rc::Rc;

use crate::{
    geometry::{Normal, Position, Tangent, Texcoord},
    mesh::Mesh,
    rtx_extensions::RtxExtensions,
};
use vk_utils::{device_context::DeviceContext, queue::CommandQueue};

use crate::{gpu_resource::GpuResource, gpu_resource_cache::GpuResourceCache};

pub struct MeshResource {
    pub indices: Vec<u32>,
    pub vertices: Vec<Position>,
    pub normals: Vec<Normal>,
    pub tangents: Vec<Tangent>,
    pub tex_coords: Vec<Texcoord>,
}

impl MeshResource {
    pub fn new(
        indices: Vec<u32>,
        vertices: Vec<Position>,
        normals: Vec<Normal>,
        tangents: Vec<Tangent>,
        tex_coords: Vec<Texcoord>,
    ) -> Self {
        Self {
            indices,
            vertices,
            normals,
            tangents,
            tex_coords,
        }
    }
}

impl GpuResource for MeshResource {
    type Item = Mesh;

    fn prepare(
        &self,
        device: Rc<DeviceContext>,
        rtx: &RtxExtensions,
        queue: Rc<CommandQueue>,
        _: &GpuResourceCache,
    ) -> Self::Item {
        // Turn Cpu mesh into Gpu mesh
        Mesh::new(
            device,
            rtx,
            queue,
            &self.indices,
            &self.vertices,
            &self.normals,
            &self.tangents,
            &self.tex_coords,
        )
    }
}
