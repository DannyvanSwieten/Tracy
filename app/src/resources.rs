use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use renderer::context::RtxContext;
use renderer::gpu_scene::{GpuTexture, Mesh, MeshAddress};
use vk_utils::device_context::DeviceContext;

use crate::image_resource::TextureImageData;
use crate::mesh_resource;
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

pub struct GpuResourceCache {
    pub meshes: HashMap<usize, Mesh>,
    pub mesh_addresses: HashMap<usize, MeshAddress>,
    pub textures: HashMap<usize, GpuTexture>,
    pub samplers: HashMap<usize, ash::vk::Sampler>,
}

impl GpuResourceCache {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            mesh_addresses: HashMap::new(),
            textures: HashMap::new(),
            samplers: HashMap::new(),
        }
    }

    pub fn add_mesh(
        &mut self,
        device: &DeviceContext,
        rtx: &RtxContext,
        mesh: &Arc<Resource<mesh_resource::MeshResource>>,
    ) -> &<mesh_resource::MeshResource as GpuResource>::Item {
        let prepared = self.meshes.get(&mesh.uid());
        if prepared.is_none() {
            self.meshes.insert(mesh.uid(), mesh.prepare(device, rtx));
        }
        self.meshes.get(&mesh.uid()).unwrap()
    }

    pub fn add_texture(&mut self, device: &DeviceContext, texture: &TextureImageData, id: usize) {
        let image = device.image_2d(
            texture.width,
            texture.height,
            texture.format,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
        );

        let buffer = device.buffer(
            texture.pixels.len() as u64,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
            ash::vk::BufferUsageFlags::TRANSFER_SRC,
        );

        device
            .graphics_queue()
            .unwrap()
            .begin(|command_buffer_handle| {
                command_buffer_handle.copy_buffer_to_image_2d(&buffer, &image);
                command_buffer_handle
            });

        let view_info = *ash::vk::ImageViewCreateInfo::builder()
            .format(texture.format)
            .image(*image.vk_image())
            .subresource_range(
                *ash::vk::ImageSubresourceRange::builder()
                    .layer_count(1)
                    .level_count(1)
                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR),
            );

        let image_view = unsafe {
            device
                .vk_device()
                .create_image_view(&view_info, None)
                .expect("Image view creation failed")
        };

        self.textures.insert(id, GpuTexture { image_view, image });
    }

    pub fn buffer_addresses(&self, id: usize) -> &MeshAddress {
        &self.mesh_addresses.get(&id).unwrap()
    }
}

unsafe impl Send for GpuResourceCache {}
