use glm::{Mat4, Vec3};

use crate::{instance::Instance, material::Material, mesh::Mesh, uid_object::UidObject};
use std::sync::Arc;

pub struct Shape {
    mesh: Arc<UidObject<Mesh>>,
    instances: Vec<Instance>,
}

impl Shape {
    pub fn new(mesh: Arc<UidObject<Mesh>>) -> Shape {
        Self {
            mesh,
            instances: Vec::new(),
        }
    }

    pub fn create_instance(&mut self, material: Arc<UidObject<Material>>, transform: &Mat4) {
        self.instances
            .push(Instance::new(self.mesh.clone(), material, transform))
    }
    pub fn mesh(&self) -> &Arc<UidObject<Mesh>> {
        &self.mesh
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }
}
