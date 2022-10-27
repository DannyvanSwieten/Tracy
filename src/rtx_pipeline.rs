use std::rc::Rc;

use ash::vk::{
    BufferUsageFlags, DeferredOperationKHR, MemoryPropertyFlags, Pipeline, PipelineCache,
    PipelineShaderStageCreateInfo, RayTracingPipelineCreateInfoKHR,
    RayTracingShaderGroupCreateInfoKHR, RayTracingShaderGroupTypeKHR, ShaderModuleCreateInfo,
    ShaderStageFlags, StridedDeviceAddressRegionKHR, SHADER_UNUSED_KHR,
};
use vk_utils::{
    buffer_resource::BufferResource, device_context::DeviceContext, shader_library::load_spirv,
};

use crate::{descriptor_sets::RTXDescriptorSets, rtx_extensions::RtxExtensions};

pub struct RtxPipeline {
    pub descriptor_sets: RTXDescriptorSets,
    pub pipeline: Pipeline,
    pub stride_addresses: Vec<StridedDeviceAddressRegionKHR>,
    pub shader_binding_table: BufferResource,
}

impl RtxPipeline {
    pub fn new(device: Rc<DeviceContext>, rtx: Rc<RtxExtensions>, max_sets: u32) -> Self {
        let descriptor_sets = RTXDescriptorSets::new(device.clone(), max_sets);

        let dir = std::env::current_exe()
            .expect("current dir check failed")
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("shaders")
            .join("simple_pipeline");
        unsafe {
            let code = load_spirv(dir.join("ray_gen.rgen.spv").to_str().unwrap());
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let gen = device
                .handle()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray generation shader compilation failed");
            let code = load_spirv(dir.join("closest_hit.rchit.spv").to_str().unwrap());
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let chit = device
                .handle()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray closest hit shader compilation failed");
            let code = load_spirv(dir.join("ray_miss.rmiss.spv").to_str().unwrap());
            let shader_module_info = ShaderModuleCreateInfo::builder().code(&code);
            let miss = device
                .handle()
                .create_shader_module(&shader_module_info, None)
                .expect("Ray miss shader compilation failed");

            let shader_groups = vec![
                // group0 = [ raygen ]
                *RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(RayTracingShaderGroupTypeKHR::GENERAL)
                    .general_shader(0)
                    .closest_hit_shader(SHADER_UNUSED_KHR)
                    .any_hit_shader(SHADER_UNUSED_KHR)
                    .intersection_shader(SHADER_UNUSED_KHR),
                // group1 = [ chit ]
                *RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP)
                    .general_shader(SHADER_UNUSED_KHR)
                    .closest_hit_shader(1)
                    .any_hit_shader(SHADER_UNUSED_KHR)
                    .intersection_shader(SHADER_UNUSED_KHR),
                // group2 = [ miss ]
                *RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(RayTracingShaderGroupTypeKHR::GENERAL)
                    .general_shader(2)
                    .closest_hit_shader(SHADER_UNUSED_KHR)
                    .any_hit_shader(SHADER_UNUSED_KHR)
                    .intersection_shader(SHADER_UNUSED_KHR),
            ];

            let shader_stages = vec![
                *PipelineShaderStageCreateInfo::builder()
                    .stage(ShaderStageFlags::RAYGEN_KHR)
                    .module(gen)
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap()),
                *PipelineShaderStageCreateInfo::builder()
                    .stage(ShaderStageFlags::CLOSEST_HIT_KHR)
                    .module(chit)
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap()),
                *PipelineShaderStageCreateInfo::builder()
                    .stage(ShaderStageFlags::MISS_KHR)
                    .module(miss)
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap()),
            ];

            let infos = [*RayTracingPipelineCreateInfoKHR::builder()
                .stages(&shader_stages)
                .groups(&shader_groups)
                .max_pipeline_ray_recursion_depth(2)
                .layout(descriptor_sets.pipeline_layout)];

            let pipeline = rtx
                .pipeline_ext()
                .create_ray_tracing_pipelines(
                    DeferredOperationKHR::null(),
                    PipelineCache::null(),
                    &infos,
                    None,
                )
                .expect("Raytracing pipeline creation failed")[0];

            let group_count = 3;
            let properties = rtx.pipeline_properties();
            let aligned_group_size = properties.shader_group_handle_size
                + (properties.shader_group_base_alignment - properties.shader_group_handle_size);
            let table_size = (aligned_group_size * group_count) as usize;
            let table_data: Vec<u8> = rtx
                .pipeline_ext()
                .get_ray_tracing_shader_group_handles(pipeline, 0, group_count, table_size)
                .expect("Get raytracing shader group handles failed");

            let mut shader_binding_table = BufferResource::new(
                device,
                table_size as u64,
                MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
                BufferUsageFlags::TRANSFER_SRC
                    | BufferUsageFlags::SHADER_BINDING_TABLE_KHR
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            );

            shader_binding_table.copy_aligned_to(
                &table_data,
                Some(properties.shader_group_handle_size as usize),
                aligned_group_size as usize,
            );

            let ray_gen_address = *StridedDeviceAddressRegionKHR::builder()
                .size(aligned_group_size.into())
                .stride(aligned_group_size.into())
                .device_address(shader_binding_table.device_address());

            let closest_hit_address = *StridedDeviceAddressRegionKHR::builder()
                .size(aligned_group_size.into())
                .stride(aligned_group_size.into())
                .device_address(shader_binding_table.device_address() + aligned_group_size as u64);

            let miss_address = *StridedDeviceAddressRegionKHR::builder()
                .size(aligned_group_size.into())
                .stride(aligned_group_size.into())
                .device_address(
                    shader_binding_table.device_address() + aligned_group_size as u64 * 2,
                );

            let callable_address = *StridedDeviceAddressRegionKHR::builder()
                .size(0)
                .stride(0)
                .device_address(0);

            let stride_addresses = vec![
                ray_gen_address,
                miss_address,
                closest_hit_address,
                callable_address,
            ];

            Self {
                descriptor_sets,
                pipeline,
                stride_addresses,
                shader_binding_table,
            }
        }
    }
}
