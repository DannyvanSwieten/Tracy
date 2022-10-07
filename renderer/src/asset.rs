use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct GpuObject<T> {
    id: usize,
    data: T,
}

impl<T> GpuObject<T> {
    pub fn new(id: usize, data: T) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

impl<T> Deref for GpuObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for GpuObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
