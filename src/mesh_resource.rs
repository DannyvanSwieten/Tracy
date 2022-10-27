use crate::geometry::{Normal, Position, Tangent, Texcoord};

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
