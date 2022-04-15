use glm::Vec3;

use crate::{
    geometry::Position, gpu_scene::Mesh, instance::Instance, material::Material, resource::Resource,
};
use std::sync::Arc;

pub struct Shape {
    mesh: Arc<Resource<Mesh>>,
    instances: Vec<Instance>,
}

impl Shape {
    pub fn new(mesh: Arc<Resource<Mesh>>) -> Shape {
        Self {
            mesh,
            instances: Vec::new(),
        }
    }

    pub fn create_instance(
        &mut self,
        material: Arc<Resource<Material>>,
        position: &Position,
        scale: &Vec3,
        rotation: &Vec3,
    ) {
        self.instances.push(Instance::new(
            self.mesh.clone(),
            material,
            position,
            scale,
            rotation,
        ))
    }
    pub fn mesh(&self) -> &Arc<Resource<Mesh>> {
        &self.mesh
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }
}
