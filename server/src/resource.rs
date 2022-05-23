use std::{any::TypeId, ops::Deref, rc::Rc, sync::Arc};

use renderer::context::RtxContext;
use vk_utils::{device_context::DeviceContext, queue::CommandQueue};

use crate::resources::GpuResourceCache;

pub trait GpuResource {
    type Item;

    fn prepare(
        &self,
        device: Rc<DeviceContext>,
        rtx: &RtxContext,
        queue: Rc<CommandQueue>,
        cache: &GpuResourceCache,
    ) -> Self::Item;
}

pub struct Resource<T>
where
    T: GpuResource,
{
    uid: usize,
    origin: String,
    name: String,
    item: T,
    type_id: TypeId,
}

impl<T: GpuResource> Deref for Resource<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T: GpuResource + 'static> Resource<T> {
    pub fn new(uid: usize, origin: &str, name: &str, data: T) -> Self {
        Self {
            uid,
            item: data,
            origin: origin.to_string(),
            name: name.to_string(),
            type_id: TypeId::of::<T>(),
        }
    }

    pub fn uid(&self) -> usize {
        self.uid
    }

    pub fn origin(&self) -> &str {
        &self.origin
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn item(&self) -> &T {
        &self.item
    }

    pub fn type_id(&self) -> &TypeId {
        &self.type_id
    }

    pub fn prepare(
        &self,
        device: Rc<DeviceContext>,
        rtx: &RtxContext,
        queue: Rc<CommandQueue>,
        cache: &GpuResourceCache,
    ) -> Arc<renderer::resource::Resource<T::Item>> {
        Arc::new(renderer::resource::Resource::new(
            self.uid,
            self.item.prepare(device, rtx, queue, cache),
        ))
    }
}
