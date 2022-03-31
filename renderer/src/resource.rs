use vk_utils::device_context::DeviceContext;

use crate::context::RtxContext;

pub trait GpuResource {
    type Item;

    fn prepare(&self, device: &DeviceContext, rtx: &RtxContext) -> Self::Item;
}

pub struct Resource<T>
where
    T: GpuResource,
{
    uid: usize,
    item: std::sync::Arc<T>,
}

impl<T: GpuResource> Resource<T> {
    pub fn new(uid: usize, data: T) -> Self {
        Self {
            uid,
            item: std::sync::Arc::new(data),
        }
    }

    pub fn prepare(&self, device: &DeviceContext, rtx: &RtxContext) -> T::Item {
        self.item.prepare(device, rtx)
    }
}
