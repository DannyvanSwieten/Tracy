use std::sync::Arc;

use nalgebra_glm::{vec2, vec3};
use renderer::geometry::{Normal, Position, Tangent, Texcoord};

use crate::{resource::Resource, resources::Resources};

use super::mesh_resource::MeshResource;

pub struct SurfaceBuilder {
    mesh_builder: MeshBuilder,
    indices: Vec<u32>,
    positions: Vec<Position>,
    normals: Vec<Normal>,
    tangents: Vec<Tangent>,
    texcoords: Vec<Texcoord>,
}

impl SurfaceBuilder {
    pub(crate) fn new(mesh_builder: MeshBuilder) -> Self {
        SurfaceBuilder {
            mesh_builder,
            indices: Vec::new(),
            positions: Vec::new(),
            normals: Vec::new(),
            tangents: Vec::new(),
            texcoords: Vec::new(),
        }
    }

    pub fn calculate_normal(self) -> Self {
        self
    }

    pub fn with_vertices(mut self, vertices: Vec<Position>) -> Self {
        self.positions = vertices;
        self
    }

    pub fn with_normals(mut self, normals: Vec<Normal>) -> Self {
        self.normals = normals;
        self
    }

    pub fn with_tangents(mut self, tangents: Vec<Tangent>) -> Self {
        self.tangents = tangents;
        self
    }
    pub fn with_texcoords(mut self, texcoords: Vec<Texcoord>) -> Self {
        self.texcoords = texcoords;
        self
    }

    pub fn with_indices(mut self, indices: Vec<u32>) -> Self {
        self.indices = indices;
        self
    }

    pub fn done_face(self) -> MeshBuilder {
        self.mesh_builder.add_surface(
            self.indices,
            self.positions,
            self.normals,
            self.tangents,
            self.texcoords,
        )
    }

    pub fn next(self) -> Self {
        self.done_face().create_surface()
    }

    pub fn done(
        self,
    ) -> (
        Vec<u32>,
        Vec<Position>,
        Vec<Normal>,
        Vec<Tangent>,
        Vec<Texcoord>,
    ) {
        let result = self.done_face();

        (
            result.indices,
            result.positions,
            result.normals,
            result.tangents,
            result.texcoords,
        )
    }
}

#[derive(Default)]
pub struct MeshBuilder {
    indices: Vec<u32>,
    positions: Vec<Position>,
    normals: Vec<Normal>,
    tangents: Vec<Tangent>,
    texcoords: Vec<Texcoord>,
}

impl MeshBuilder {
    pub fn create_surface(self) -> SurfaceBuilder {
        SurfaceBuilder::new(self)
    }

    pub(crate) fn add_surface(
        mut self,
        indices: Vec<u32>,
        positions: Vec<Position>,
        normals: Vec<Normal>,
        tangents: Vec<Tangent>,
        texcoords: Vec<Texcoord>,
    ) -> Self {
        self.indices.extend(indices);
        self.positions.extend(positions);
        self.normals.extend(normals);
        self.tangents.extend(tangents);
        self.texcoords.extend(texcoords);
        self
    }
}

pub fn create_plane(
    resource_cache: &mut Resources,
    width: f32,
    height: f32,
) -> Arc<Resource<MeshResource>> {
    let left = -width / 2.0;
    let right = width / 2.0;
    let bttm = -height / 2.0;
    let top = height / 2.0;
    let (indices, positions, normals, tangents, texcoords) = MeshBuilder::default()
        .create_surface()
        .with_vertices(vec![
            vec3(left, 0.0, bttm),
            vec3(left, 0.0, top),
            vec3(right, 0.0, top),
            vec3(right, 0.0, bttm),
        ])
        .with_indices(vec![0, 1, 2, 0, 2, 3])
        .with_normals(vec![
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        ])
        .with_tangents(vec![
            vec3(1.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
        ])
        .with_texcoords(vec![vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.5, 1.0)])
        .done();

    resource_cache.add_mesh(
        "Internal",
        "X/Z Plane",
        MeshResource::new(indices, positions, normals, tangents, texcoords),
    )
}

pub fn create_triangle(resource_cache: &mut Resources) -> Arc<Resource<MeshResource>> {
    let (indices, positions, normals, tangents, texcoords) = MeshBuilder::default()
        .create_surface()
        .with_vertices(vec![
            vec3(-1.0, -1.0, 0.0),
            vec3(1.0, -1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        ])
        .with_indices(vec![0, 1, 2])
        .with_normals(vec![
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 0.0, 1.0),
        ])
        .with_tangents(vec![
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        ])
        .with_texcoords(vec![vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.5, 1.0)])
        .done();

    resource_cache.add_mesh(
        "Internal",
        "Triangle",
        MeshResource::new(indices, positions, normals, tangents, texcoords),
    )
}
