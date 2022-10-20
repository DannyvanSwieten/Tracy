use ash::vk::GeometryInstanceFlagsKHR;
use ash::vk::QueueFlags;
use cgmath::SquareMatrix;
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

use crate::geometry::GeometryInstance;
use crate::gpu_scene::GpuTexture;
use crate::image_resource::TextureImageData;
use crate::math::Mat4;
use crate::math::Vec4;
use crate::mesh::Mesh;
use crate::mesh::MeshAddress;
use crate::mesh_resource::MeshResource;
use crate::rtx_extensions::RtxExtensions;
use crate::scene::Scene;

pub type Handle = DefaultKey;
type Map<V> = SlotMap<Handle, V>;

pub struct Frame {
    gpu_instances: Vec<GeometryInstance>,
}

pub struct Ctx {
    device: Rc<DeviceContext>,
    rtx: RtxExtensions,
    queue: Rc<CommandQueue>,
    textures: Map<GpuTexture>,
    meshes: Map<Mesh>,
    instances: Map<MeshInstance>,
    default_material: Handle,
    materials: Map<Material2>,
}

pub struct MeshInstance {
    mesh: Handle,
    material: Handle,
    transform: Mat4,
}

impl MeshInstance {
    fn new(mesh: Handle, material: Handle) -> Self {
        Self {
            mesh,
            material,
            transform: Mat4::identity(),
        }
    }

    pub fn mesh(&self) -> Handle {
        self.mesh
    }

    pub fn material(&self) -> Handle {
        self.material
    }

    pub fn transform(&self) -> &Mat4 {
        &self.transform
    }
}
pub struct Material2 {
    pub base_color: Vec4,
    pub emission: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub sheen: f32,
    pub clear_coat: f32,
    pub base_color_texture: Option<Handle>,
    pub metallic_roughness_texture: Option<Handle>,
    pub normal_texture: Option<Handle>,
    pub emission_texture: Option<Handle>,
}

impl Material2 {
    pub fn new() -> Self {
        Self {
            base_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            emission: Vec4::new(0.0, 0.0, 0.0, 0.0),
            roughness: 0.5,
            metallic: 0.0,
            sheen: 0.0,
            clear_coat: 0.0,
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_texture: None,
            emission_texture: None,
        }
    }
}

impl Ctx {
    pub fn new(device: Rc<DeviceContext>) -> Self {
        let rtx = RtxExtensions::new(&device);
        let queue = Rc::new(CommandQueue::new(device.clone(), QueueFlags::GRAPHICS));
        let mut instance = Self {
            device: device.clone(),
            rtx,
            textures: Map::new(),
            meshes: Map::new(),
            instances: Map::new(),
            default_material: Handle::default(),
            materials: Map::new(),
            queue,
        };

        let default_material = instance.create_material();
        instance.default_material = default_material;
        instance
    }

    pub fn create_material(&mut self) -> Handle {
        self.materials.insert(Material2::new())
    }

    pub fn material_mut(&mut self, material: Handle) -> Option<&mut Material2> {
        self.materials.get_mut(material)
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

    pub fn create_instance(&mut self, mesh: Handle) -> Handle {
        self.instances
            .insert(MeshInstance::new(mesh, self.default_material))
    }

    pub fn build_frame(&self, scene: &Scene) -> Frame {
        let mut geometry_map = HashMap::new();
        let mut geometries = Vec::new();
        for (index, (key, geometry)) in self.meshes.iter().enumerate() {
            geometry_map.insert(key, index);
            geometries.push((geometry, MeshAddress::new(geometry)));
        }

        let mut gpu_instances = Vec::new();
        for (instance_id, key) in scene.instances().iter().enumerate() {
            if let Some(instance) = self.instances.get(*key) {
                let geometry_index = *geometry_map.get(key).unwrap();
                let (mesh, addresses) = &geometries[geometry_index];
                gpu_instances.push(GeometryInstance::new(
                    instance_id as u32,
                    0xff,
                    0,
                    GeometryInstanceFlagsKHR::FORCE_OPAQUE,
                    mesh.blas.address(),
                    instance.transform(),
                ));
            }
        }

        Frame { gpu_instances }
    }
}
