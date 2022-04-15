use std::{ops::Deref, sync::Arc};

use renderer::context::RtxContext;
use vk_utils::device_context::DeviceContext;

use crate::resources::GpuResourceCache;

pub trait GpuResource {
    type Item;

    fn prepare(
        &self,
        device: &DeviceContext,
        rtx: &RtxContext,
        cache: &GpuResourceCache,
    ) -> Self::Item;
}

pub struct Resource<T>
where
    T: GpuResource,
{
    uid: usize,
    item: T,
}

impl<T: GpuResource> Deref for Resource<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T: GpuResource> Resource<T> {
    pub fn new(uid: usize, data: T) -> Self {
        Self { uid, item: data }
    }

    pub fn uid(&self) -> usize {
        self.uid
    }

    pub fn item(&self) -> &T {
        &self.item
    }

    pub fn prepare(
        &self,
        device: &DeviceContext,
        rtx: &RtxContext,
        cache: &GpuResourceCache,
    ) -> Arc<renderer::resource::Resource<T::Item>> {
        Arc::new(renderer::resource::Resource::new(
            self.uid,
            self.item.prepare(device, rtx, cache),
        ))
    }
}
