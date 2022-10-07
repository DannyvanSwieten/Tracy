use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct UidObject<T> {
    id: usize,
    data: T,
}

impl<T> UidObject<T> {
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

impl<T> Deref for UidObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for UidObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
