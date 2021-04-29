use ash;
use ash::extensions::khr::{AccelerationStructure, DeferredHostOperations, RayTracingPipeline};

use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, InstanceV1_1, InstanceV1_2};
use ash::vk;
use ash::vk::{
    Buffer, CommandPoolCreateInfo, DescriptorBindingFlagsEXT, DescriptorPool,
    DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetLayout,
    DescriptorSetLayoutBinding, DescriptorSetLayoutBindingFlagsCreateInfoEXT,
    DescriptorSetLayoutCreateInfo, DescriptorType, Image, ImageView, Queue, ShaderModule,
    ShaderModuleCreateInfo, ShaderStageFlags,
};
use ash::vk::{
    ExtendsPhysicalDeviceProperties2, PhysicalDeviceRayTracingPipelinePropertiesKHR,
    RayTracingPipelineCreateInfoKHR,
};

use ash::{Device, Instance};

pub struct Renderer {
    context: Device,
    queue: Queue,
    queue_family_index: u32,
    descriptor_sets: Vec<DescriptorSet>,
    pipeline_properties: PhysicalDeviceRayTracingPipelinePropertiesKHR,
    rtx_pipeline_extension: RayTracingPipeline,
    descriptor_pool: DescriptorPool,
    descriptor_set_layout: DescriptorSetLayout,
    accumulation_image: Image,
    output_image: Image,
    output_image_view: ImageView,
}

impl Renderer {
    pub fn new(instance: &Instance) -> Self {
        unsafe {
            let pdevices = instance
                .enumerate_physical_devices()
                .expect("Physical device error");
            let (gpu, queue_family_index) = pdevices
                .iter()
                .map(|pdevice| {
                    instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, ref info)| {
                            let supports_graphics =
                                info.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS);
                            if supports_graphics {
                                Some((*pdevice, index))
                            } else {
                                None
                            }
                        })
                        .next()
                })
                .filter_map(|v| v)
                .next()
                .expect("Couldn't find suitable device.");

            queue_family_index as u32;

            let priorities = [1.0];

            let queue_info = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index as u32)
                .queue_priorities(&priorities)
                .build()];

            let device_extension_names_raw = [
                RayTracingPipeline::name().as_ptr(),
                DeferredHostOperations::name().as_ptr(),
                AccelerationStructure::name().as_ptr(),
            ];

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_info)
                .enabled_extension_names(&device_extension_names_raw);

            let context = instance
                .create_device(gpu, &device_create_info, None)
                .expect("Failed raytracing device context creation");

            let rtx_pipeline_extension = RayTracingPipeline::new(instance, &context);

            let mut pipeline_properties =
                vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
            let mut properties =
                vk::PhysicalDeviceProperties2::builder().push_next(&mut pipeline_properties);
            instance.get_physical_device_properties2(gpu, &mut properties);
            let queue = context.get_device_queue(queue_family_index as u32, 0);

            let mut result = Self {
                context,
                queue,
                queue_family_index: queue_family_index as u32,
                descriptor_sets: Vec::new(),
                rtx_pipeline_extension,
                pipeline_properties,
                descriptor_pool: DescriptorPool::null(),
                descriptor_set_layout: DescriptorSetLayout::null(),
                accumulation_image: Image::null(),
                output_image: Image::null(),
                output_image_view: ImageView::null(),
            };

            result.create_descriptor_pool();
            result.create_descriptor_set_layout();

            result
        }
    }

    fn create_descriptor_pool(&mut self) {
        unsafe {
            let sizes = [
                DescriptorPoolSize {
                    ty: DescriptorType::STORAGE_IMAGE,
                    descriptor_count: 2,
                },
                DescriptorPoolSize {
                    ty: DescriptorType::ACCELERATION_STRUCTURE_KHR,
                    descriptor_count: 1,
                },
            ];
            let descriptor_pool_create_info = DescriptorPoolCreateInfo::builder()
                .max_sets(1)
                .pool_sizes(&sizes);
            self.descriptor_pool = self
                .context
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Descriptor pool creation failed");
        }
    }

    fn create_descriptor_set_layout(&mut self) {
        unsafe {
            let bindings = [
                // acceleration structure
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(0)
                    .build(),
                // position buffer
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_BUFFER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(1)
                    .build(),
                // index buffer
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_BUFFER)
                    .stage_flags(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .binding(1)
                    .build(),
                // accumulation image
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_IMAGE)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(1)
                    .build(),
                // final image
                DescriptorSetLayoutBinding::builder()
                    .descriptor_count(1)
                    .descriptor_type(DescriptorType::STORAGE_IMAGE)
                    .stage_flags(ShaderStageFlags::RAYGEN_KHR)
                    .binding(1)
                    .build(),
            ];

            let mut binding_flags = DescriptorSetLayoutBindingFlagsCreateInfoEXT::builder()
                .binding_flags(&[
                    DescriptorBindingFlagsEXT::empty(),
                    DescriptorBindingFlagsEXT::empty(),
                    DescriptorBindingFlagsEXT::empty(),
                    DescriptorBindingFlagsEXT::empty(),
                    DescriptorBindingFlagsEXT::empty(),
                ])
                .build();

            let layout_info = DescriptorSetLayoutCreateInfo::builder()
                .bindings(&bindings)
                .push_next(&mut binding_flags);
            self.descriptor_set_layout = self
                .context
                .create_descriptor_set_layout(&layout_info, None)
                .expect("Descriptor set layout creation failed");
        }
    }

    fn load_shaders(&mut self) {}

    fn create_images_and_views(&mut self) {}
}
