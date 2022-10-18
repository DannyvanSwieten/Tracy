use std::sync::Arc;

use cgmath::Matrix;

use crate::geometry::GeometryInstance;
use crate::material::Material;
use crate::math::Mat4;
use crate::mesh::Mesh;
use crate::uid_object::Handle;

pub struct Instance {
    transform: Mat4,
    mesh: Arc<Handle<Mesh>>,
    material: Arc<Handle<Material>>,
}
impl Instance {
    pub fn new(mesh: Arc<Handle<Mesh>>, material: Arc<Handle<Material>>, transform: &Mat4) -> Self {
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
            self.transform.transpose(),
        )
    }
}
