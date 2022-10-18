use crate::{instance::Instance, material::Material, math::Mat4, mesh::Mesh, uid_object::Handle};
use std::sync::Arc;

pub struct Shape {
    mesh: Arc<Handle<Mesh>>,
    instances: Vec<Instance>,
}

impl Shape {
    pub fn new(mesh: Arc<Handle<Mesh>>) -> Shape {
        Self {
            mesh,
            instances: Vec::new(),
        }
    }

    pub fn create_instance(&mut self, material: Arc<Handle<Material>>, transform: &Mat4) {
        self.instances
            .push(Instance::new(self.mesh.clone(), material, transform))
    }
    pub fn mesh(&self) -> &Arc<Handle<Mesh>> {
        &self.mesh
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }
}
