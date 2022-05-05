use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct Resource<T> {
    id: usize,
    data: T,
}

impl<T> Resource<T> {
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

impl<T> Deref for Resource<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Resource<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
