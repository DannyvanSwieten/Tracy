use crate::ctx::Handle;

pub struct Scene {
    instances: Vec<Handle>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
        }
    }

    pub fn instances(&self) -> &[Handle] {
        &self.instances
    }

    pub fn add_instance(&mut self, instance: Handle) {
        self.instances.push(instance)
    }
}
