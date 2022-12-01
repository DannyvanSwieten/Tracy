use cgmath::SquareMatrix;

use crate::{
    ctx::Handle,
    math::{Mat4, Vec3},
};

pub struct MeshInstance {
    mesh: Handle,
    material: Handle,
    transform: Mat4,
}

impl MeshInstance {
    pub fn new(mesh: Handle, material: Handle) -> Self {
        Self {
            mesh,
            material,
            transform: Mat4::identity(),
        }
    }

    pub fn mesh(&self) -> Handle {
        self.mesh
    }

    pub fn set_material(&mut self, material: Handle) -> &mut Self {
        self.material = material;
        self
    }

    pub fn material(&self) -> Handle {
        self.material
    }

    pub fn transform(&self) -> &Mat4 {
        &self.transform
    }

    pub fn scale(&mut self, scale: &Vec3) -> &mut Self {
        self.transform = self.transform * Mat4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        self
    }

    pub fn translate(&mut self, translation: &Vec3) -> &mut Self {
        self.transform = self.transform * Mat4::from_translation(*translation);
        self
    }

    pub fn rotate(&mut self, rotation: &Vec3) -> &mut Self {
        self.transform = self.transform * Mat4::from_angle_x(cgmath::Deg(rotation.x));
        self.transform = self.transform * Mat4::from_angle_y(cgmath::Deg(rotation.y));
        self.transform = self.transform * Mat4::from_angle_z(cgmath::Deg(rotation.z));
        self
    }
}
