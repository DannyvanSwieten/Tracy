use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::resource::{GpuResource, Resource};
#[derive(Default)]
pub struct Resources {
    pub data: HashMap<TypeId, HashMap<usize, Arc<dyn Any>>>,
}

impl Resources {
    pub fn add<T: 'static + GpuResource>(&mut self, resource: T) -> Arc<Resource<T>> {
        let type_id = TypeId::of::<T>();
        if let Some(map) = self.data.get_mut(&type_id) {
            let any: Arc<dyn Any> = Arc::new(Arc::new(Resource::<T>::new(0, resource)));
            map.insert(0, any.clone());
            map.get(&0)
                .unwrap()
                .downcast_ref::<Arc<Resource<T>>>()
                .unwrap()
                .clone()
        } else {
            self.data.insert(type_id, HashMap::new());
            self.add(resource)
        }
    }

    //pub fn get<T: GpuResource>(&self, id: usize) -> &Resource<T> {}
}

unsafe impl Send for Resources {}
