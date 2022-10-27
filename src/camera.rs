use cgmath::{perspective, Deg, Rad, SquareMatrix};

use crate::math::{Mat4, Real, Vec3};

#[derive(Clone, Copy)]
pub struct Camera {
    fov: Real,
    z_near: Real,
    z_far: Real,
    transform: Mat4,
}

impl Camera {
    pub fn new(fov: Real, z_near: Real, z_far: Real) -> Self {
        Self {
            fov,
            z_near,
            z_far,
            transform: Mat4::identity(),
        }
    }

    pub fn transform(&mut self, transform: Mat4) {
        self.transform = self.transform * transform
    }

    pub fn translate(&mut self, t: Vec3) {
        self.transform = self.transform * Mat4::from_translation(t)
    }

    pub fn view_matrix(&self) -> Mat4 {
        self.transform.invert().unwrap()
    }

    pub fn projection_matrix(&self, aspect_ratio: Real) -> Mat4 {
        perspective(Deg(self.fov), aspect_ratio, self.z_near, self.z_far)
            .invert()
            .unwrap()
    }
}
