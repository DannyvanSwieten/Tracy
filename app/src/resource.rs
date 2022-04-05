use renderer::context::RtxContext;
use vk_utils::device_context::DeviceContext;

pub trait GpuResource {
    type Item;

    fn prepare(&self, device: &DeviceContext, rtx: &RtxContext) -> Self::Item;
}

pub struct Resource<T>
where
    T: GpuResource,
{
    uid: usize,
    item: T,
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

    pub fn prepare(&self, device: &DeviceContext, rtx: &RtxContext) -> T::Item {
        self.item.prepare(device, rtx)
    }
}
