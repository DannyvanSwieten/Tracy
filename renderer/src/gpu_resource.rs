use std::{any::TypeId, ops::Deref, rc::Rc, sync::Arc};

use vk_utils::{device_context::DeviceContext, queue::CommandQueue};

use crate::{asset::GpuObject, context::RtxExtensions, gpu_resource_cache::GpuResourceCache};

pub trait GpuResource {
    type Item;

    fn prepare(
        &self,
        device: Rc<DeviceContext>,
        rtx: &RtxExtensions,
        queue: Rc<CommandQueue>,
        cache: &GpuResourceCache,
    ) -> Self::Item;
}

pub struct CpuResource<T>
where
    T: GpuResource,
{
    uid: usize,
    origin: String,
    name: String,
    item: T,
    type_id: TypeId,
}

impl<T: GpuResource> Deref for CpuResource<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T: GpuResource + 'static> CpuResource<T> {
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
        rtx: &RtxExtensions,
        queue: Rc<CommandQueue>,
        cache: &GpuResourceCache,
    ) -> Arc<GpuObject<T::Item>> {
        Arc::new(GpuObject::new(
            self.uid,
            self.item.prepare(device, rtx, queue, cache),
        ))
    }
}
