use glm::{Mat4, Vec3};

use crate::{asset::GpuObject, instance::Instance, material::Material, mesh::Mesh};
use std::sync::Arc;

pub struct Shape {
    mesh: Arc<GpuObject<Mesh>>,
    instances: Vec<Instance>,
}

impl Shape {
    pub fn new(mesh: Arc<GpuObject<Mesh>>) -> Shape {
        Self {
            mesh,
            instances: Vec::new(),
        }
    }

    pub fn create_instance(&mut self, material: Arc<GpuObject<Material>>, transform: &Mat4) {
        self.instances
            .push(Instance::new(self.mesh.clone(), material, transform))
    }
    pub fn mesh(&self) -> &Arc<GpuObject<Mesh>> {
        &self.mesh
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }
}
