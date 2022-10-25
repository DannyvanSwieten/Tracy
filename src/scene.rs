use crate::{camera::Camera, ctx::Handle};

pub struct Scene {
    instances: Vec<Handle>,
    camera: Camera,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
            camera: Camera::new(1.13, 0.01, 1000.0),
        }
    }

    pub fn instances(&self) -> &[Handle] {
        &self.instances
    }

    pub fn add_instance(&mut self, instance: Handle) {
        self.instances.push(instance)
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
