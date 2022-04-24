use std::sync::Arc;

use glm::{vec4, Mat4, Mat4x3, Vec3};

use crate::geometry::GeometryInstance;
use crate::resource::Resource;
use crate::{gpu_scene::Mesh, material::Material};

pub struct Instance {
    transform: Mat4,
    mesh: Arc<Resource<Mesh>>,
    material: Arc<Resource<Material>>,
}
impl Instance {
    pub fn new(
        mesh: Arc<Resource<Mesh>>,
        material: Arc<Resource<Material>>,
        transform: &Mat4,
    ) -> Self {
        Self {
            material,
            mesh,
            transform: *transform,
        }
    }

    pub fn transform(&self) -> &Mat4 {
        &self.transform
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn gpu_instance(&self, instance_id: u32) -> GeometryInstance {
        GeometryInstance::new(
            instance_id,
            0xff,
            0,
            ash::vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE,
            self.mesh.blas.address(),
            self.transform.transpose().remove_column(3),
        )
    }
}
