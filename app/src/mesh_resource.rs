use renderer::geometry::{Normal, Position, Tangent, Texcoord};

pub struct MeshResource {
    indices: Vec<u32>,
    positions: Vec<Position>,
    normals: Vec<Normal>,
    tangents: Vec<Tangent>,
    tex_coords: Vec<Texcoord>,
}
