use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static GLOBAL_RESOURCE_ID: AtomicUsize = AtomicUsize::new(0);

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

pub struct ResourceBuilder {}

impl ResourceBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create<T>(&mut self, value: T) -> Rc<Resource<T>> {
        Rc::new(Resource::new(Self::next_uid(), value))
    }

    fn next_uid() -> usize {
        GLOBAL_RESOURCE_ID.fetch_add(1, Ordering::SeqCst)
    }
}
