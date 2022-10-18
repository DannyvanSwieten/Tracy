use ash::vk::QueueFlags;
use slotmap::DefaultKey;
use slotmap::SlotMap;
use std::collections::HashMap;
use std::rc::Rc;
use vk_utils::buffer_resource::BufferResource;
use vk_utils::command_buffer::CommandBuffer;
use vk_utils::device_context::DeviceContext;
use vk_utils::image2d_resource::Image2DResource;
use vk_utils::image_resource::ImageResource;
use vk_utils::queue::CommandQueue;

use crate::gpu_scene::GpuTexture;
use crate::image_resource::TextureImageData;
use crate::mesh::Mesh;
use crate::mesh_resource::MeshResource;
use crate::rtx_extensions::RtxExtensions;

type Handle = DefaultKey;
type Map<V> = SlotMap<Handle, V>;

pub struct Ctx {
    device: Rc<DeviceContext>,
    rtx: RtxExtensions,
    queue: Rc<CommandQueue>,
    textures: Map<GpuTexture>,
    meshes: Map<Mesh>,
    instances: HashMap<Handle, Map<MeshInstance>>,
}

struct MeshInstance {
    mesh: Handle,
}

impl Ctx {
    pub fn new(device: Rc<DeviceContext>) -> Self {
        let rtx = RtxExtensions::new(&device);
        let queue = Rc::new(CommandQueue::new(device.clone(), QueueFlags::GRAPHICS));
        Self {
            device: device.clone(),
            rtx,
            textures: Map::new(),
            meshes: Map::new(),
            instances: HashMap::new(),
            queue,
        }
    }

    pub fn create_mesh(&mut self, mesh: &MeshResource) -> Handle {
        let m = Mesh::new(
            self.device.clone(),
            &self.rtx,
            self.queue.clone(),
            &mesh.indices,
            &mesh.vertices,
            &mesh.normals,
            &mesh.tangents,
            &mesh.tex_coords,
        );

        let key = self.meshes.insert(m);
        self.instances.insert(key, Map::new());
        key
    }

    pub fn create_texture(&mut self, data: &TextureImageData) {
        let mut image = Image2DResource::new(
            self.device.clone(),
            data.width,
            data.height,
            data.format,
            ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let mut buffer = BufferResource::new(
            self.device.clone(),
            data.pixels.len() as u64,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
            ash::vk::BufferUsageFlags::TRANSFER_SRC,
        );

        buffer.upload(&data.pixels);

        let mut command_buffer = CommandBuffer::new(self.device.clone(), self.queue.clone());
        command_buffer.begin();
        command_buffer
            .image_resource_transition(&mut image, ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL);

        command_buffer.copy_buffer_to_image(&buffer, &mut image);

        command_buffer
            .image_resource_transition(&mut image, ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        command_buffer.submit();

        let view_info = *ash::vk::ImageViewCreateInfo::builder()
            .format(data.format)
            .view_type(ash::vk::ImageViewType::TYPE_2D)
            .image(image.handle())
            .subresource_range(
                *ash::vk::ImageSubresourceRange::builder()
                    .layer_count(1)
                    .level_count(1)
                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR),
            );

        let image_view = unsafe {
            self.device
                .handle()
                .create_image_view(&view_info, None)
                .expect("Image view creation failed")
        };

        self.textures.insert(GpuTexture { image, image_view });
    }

    pub fn create_instance(&mut self, mesh: &Handle) -> Handle {
        if let Some(instances) = self.instances.get_mut(mesh) {
            instances.insert(MeshInstance { mesh: *mesh })
        } else {
            panic!()
        }
    }
}

pub struct Actor {
    instance: Handle,
}

impl Actor {}
