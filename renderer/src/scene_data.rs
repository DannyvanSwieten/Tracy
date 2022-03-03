use crate::context::RtxContext;
use crate::geometry::{
    BottomLevelAccelerationStructure, GeometryInstance, GeometryOffset,
    TopLevelAccelerationStructure, Vertex,
};
use crate::scene::{Material, Scene};
use vk_utils::buffer_resource::BufferResource;
use vk_utils::device_context::DeviceContext;
use vk_utils::image_resource::Image2DResource;

use ash::vk::{BufferUsageFlags, GeometryInstanceFlagsKHR, MemoryPropertyFlags};

pub struct SceneData {
    pub vertex_buffer: BufferResource,
    pub normal_buffer: BufferResource,
    pub index_buffer: BufferResource,
    pub tex_coord_buffer: BufferResource,
    pub offset_buffer: BufferResource,
    pub material_buffer: BufferResource,
    pub address_buffer: BufferResource,
    pub bottom_level_acceleration_structures: Vec<BottomLevelAccelerationStructure>,
    pub top_level_acceleration_structure: TopLevelAccelerationStructure,
    pub images: Vec<Image2DResource>,
    pub image_views: Vec<ash::vk::ImageView>,
    pub samplers: Vec<ash::vk::Sampler>,
}

impl SceneData {
    pub fn new(device: &DeviceContext, rtx: &RtxContext, scene: &Scene) -> Self {
        let geometry = scene.geometry_buffer();
        let mut vertex_buffer = device.buffer(
            (geometry.vertices().len() * std::mem::size_of::<Vertex>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        vertex_buffer.copy_to(geometry.vertices());

        let mut index_buffer = device.buffer(
            (geometry.indices().len() * std::mem::size_of::<u32>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        );

        let mut normal_buffer = device.buffer(
            (geometry.normals().len() * std::mem::size_of::<glm::Vec3>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS | BufferUsageFlags::STORAGE_BUFFER,
        );

        normal_buffer.copy_to(geometry.normals());

        index_buffer.copy_to(geometry.indices());

        let mut tex_coord_buffer = device.buffer(
            (geometry.vertices().len() * std::mem::size_of::<nalgebra_glm::Vec2>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS | BufferUsageFlags::STORAGE_BUFFER,
        );

        tex_coord_buffer.copy_to(geometry.tex_coords());

        let mut offset_buffer = device.buffer(
            (scene.geometry_offsets().len() * std::mem::size_of::<GeometryOffset>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        offset_buffer.copy_to(scene.geometry_offsets());

        let mut material_buffer = device.buffer(
            (scene.materials().len() * std::mem::size_of::<Material>()) as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        material_buffer.copy_to(scene.materials());

        let mut address_buffer = device.buffer(
            48 as u64,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::UNIFORM_BUFFER,
        );

        address_buffer.copy_to(&[
            vertex_buffer.device_address(),
            normal_buffer.device_address(),
            index_buffer.device_address(),
            tex_coord_buffer.device_address(),
            offset_buffer.device_address(),
            material_buffer.device_address(),
        ]);

        let bottom_level_acceleration_structures: Vec<BottomLevelAccelerationStructure> = scene
            .geometry_buffer_views()
            .iter()
            .map(|view| {
                BottomLevelAccelerationStructure::new(
                    &device,
                    &rtx,
                    &vertex_buffer,
                    view.vertex_count(),
                    view.vertex_offset(),
                    &index_buffer,
                    view.index_count(),
                    view.index_offset(),
                )
            })
            .collect();

        let instances: Vec<GeometryInstance> = scene
            .geometry_instances()
            .iter()
            .enumerate()
            .map(|(i, instance)| {
                let mut ni = GeometryInstance::new(
                    i as u32,
                    0xff,
                    0,
                    GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE,
                    bottom_level_acceleration_structures[instance.geometry_id()].address(),
                );
                ni.transform = instance.transform;
                ni
            })
            .collect();

        let top_level_acceleration_structure =
            TopLevelAccelerationStructure::new(&device, &rtx, &instances);

        let images: Vec<Image2DResource> = scene
            .images()
            .iter()
            .map(|data| {
                let mut buffer = device.buffer(
                    data.pixels.len() as _,
                    ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
                    ash::vk::BufferUsageFlags::TRANSFER_SRC,
                );

                buffer.copy_to(&data.pixels);

                let mut image = device.image_2d(
                    data.width,
                    data.height,
                    data.format,
                    ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    ash::vk::ImageUsageFlags::SAMPLED | ash::vk::ImageUsageFlags::TRANSFER_DST,
                );

                device.graphics_queue().unwrap().begin(|command_buffer| {
                    command_buffer.color_image_resource_transition(
                        &mut image,
                        ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    );

                    command_buffer.copy_buffer_to_image_2d(&buffer, &image);
                    command_buffer.color_image_resource_transition(
                        &mut image,
                        ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    );
                    command_buffer
                });

                image
            })
            .collect();

        let image_views: Vec<ash::vk::ImageView> = images
            .iter()
            .map(|image| {
                let range = ash::vk::ImageSubresourceRange::builder()
                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1);
                let info_builder = ash::vk::ImageViewCreateInfo::builder()
                    .image(*image.vk_image())
                    .view_type(ash::vk::ImageViewType::TYPE_2D)
                    .format(image.format())
                    .subresource_range(*range);
                unsafe {
                    device
                        .vk_device()
                        .create_image_view(&info_builder, None)
                        .expect("Image View Creation Failed")
                }
            })
            .collect();

        let mut samplers = Vec::new();
        unsafe {
            samplers.push(
                device
                    .vk_device()
                    .create_sampler(
                        &ash::vk::SamplerCreateInfo::builder()
                            .min_filter(ash::vk::Filter::LINEAR)
                            .mag_filter(ash::vk::Filter::LINEAR)
                            .anisotropy_enable(true)
                            .max_anisotropy(8.0),
                        None,
                    )
                    .expect("Sampler creation failed"),
            );
        }

        Self {
            vertex_buffer,
            normal_buffer,
            index_buffer,
            tex_coord_buffer,
            offset_buffer,
            material_buffer,
            address_buffer,
            bottom_level_acceleration_structures,
            top_level_acceleration_structure,
            images: images,
            image_views,
            samplers,
        }
    }
}
