use ash::vk::{
    BlendFactor, BlendOp, DescriptorPool, DescriptorSet, ImageMemoryBarrier, Pipeline,
    PipelineLayout, RenderPass, Sampler, ShaderModule,
};

use vk_utils::command_buffer::CommandBuffer;
use vk_utils::device_context::DeviceContext;
use vk_utils::shader_library::load_spirv;

pub struct ImageRenderer {
    device: ash::Device,
    descriptor_pool: DescriptorPool,
    descriptor_sets: Vec<DescriptorSet>,
    vertex_shader_module: ShaderModule,
    fragment_shader_module: ShaderModule,
    pipeline_layout: PipelineLayout,
    graphics_pipeline: Pipeline,
    sampler: Sampler,

    to_shader_read_barriers: Vec<ImageMemoryBarrier>,
}

impl ImageRenderer {
    pub fn new(
        device: &DeviceContext,
        render_pass: &RenderPass,
        frame_count: usize,
        width: u32,
        height: u32,
    ) -> Self {
        let dir = std::env::current_exe()
            .expect("current dir check failed")
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("shaders");
        let buffer = load_spirv(dir.join("sampled_image.vert.spv").to_str().unwrap());
        let vertex_shader_module_create_info = ash::vk::ShaderModuleCreateInfo::builder()
            .code(&buffer)
            .build();

        let vertex_shader_module = unsafe {
            device
                .handle()
                .create_shader_module(&vertex_shader_module_create_info, None)
                .expect("Vertex Shader module creation failed")
        };

        let buffer = load_spirv(dir.join("sampled_image.frag.spv").to_str().unwrap());

        let fragment_shader_module_create_info = ash::vk::ShaderModuleCreateInfo::builder()
            .code(&buffer)
            .build();

        let fragment_shader_module = unsafe {
            device
                .handle()
                .create_shader_module(&fragment_shader_module_create_info, None)
                .expect("Vertex Shader module creation failed")
        };

        let vertex_shader_stage_create_info = ash::vk::PipelineShaderStageCreateInfo::builder()
            .stage(ash::vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader_module)
            .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();

        let fragment_shader_stage_create_info = ash::vk::PipelineShaderStageCreateInfo::builder()
            .stage(ash::vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader_module)
            .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();

        let stages = &[
            vertex_shader_stage_create_info,
            fragment_shader_stage_create_info,
        ];

        let image_sampler_binding = ash::vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(ash::vk::ShaderStageFlags::FRAGMENT)
            .build();

        let bindings = &[image_sampler_binding];
        let descriptor_set_layout_create_info =
            ash::vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

        let descriptor_set_layout = unsafe {
            device
                .handle()
                .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
                .expect("Failed to create descriptor set layout")
        };

        let layouts = &[
            descriptor_set_layout,
            descriptor_set_layout,
            descriptor_set_layout,
        ];

        let pipeline_layout_create_info = ash::vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(layouts)
            .build();

        let pipeline_layout = unsafe {
            device
                .handle()
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Pipeline layout creation failed")
        };

        let rasterization_state_create_info =
            ash::vk::PipelineRasterizationStateCreateInfo::builder()
                .polygon_mode(ash::vk::PolygonMode::FILL)
                .line_width(1.)
                .cull_mode(ash::vk::CullModeFlags::BACK)
                .front_face(ash::vk::FrontFace::COUNTER_CLOCKWISE)
                .build();

        let viewports = [ash::vk::Viewport::builder()
            .width(width as f32)
            .height(height as f32)
            .min_depth(0.)
            .max_depth(1.)
            .build()];

        let scissors = [ash::vk::Rect2D::builder()
            .offset(ash::vk::Offset2D { x: 0, y: 0 })
            .extent(ash::vk::Extent2D { width, height })
            .build()];

        let viewport_state_create_info = ash::vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors)
            .build();

        let multisample_state_create_info = ash::vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1);

        let input_assembly_state_create_info =
            ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(ash::vk::PrimitiveTopology::TRIANGLE_LIST)
                .build();

        let vertex_input_state_create_info =
            ash::vk::PipelineVertexInputStateCreateInfo::builder().build();

        let blend_attachment = [ash::vk::PipelineColorBlendAttachmentState::builder()
            .dst_color_blend_factor(BlendFactor::ONE_MINUS_SRC_ALPHA)
            .src_color_blend_factor(BlendFactor::SRC_ALPHA)
            .alpha_blend_op(BlendOp::ADD)
            .blend_enable(true)
            .color_write_mask(
                ash::vk::ColorComponentFlags::R
                    | ash::vk::ColorComponentFlags::G
                    | ash::vk::ColorComponentFlags::B
                    | ash::vk::ColorComponentFlags::A,
            )
            .build()];
        let blend_state_create_info = ash::vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(&blend_attachment)
            .build();

        let pipeline_cache_create_info = ash::vk::PipelineCacheCreateInfo::builder().build();

        let cache = unsafe {
            device
                .handle()
                .create_pipeline_cache(&pipeline_cache_create_info, None)
                .expect("Pipeline cache creation failed")
        };

        let graphics_pipeline_create_info = ash::vk::GraphicsPipelineCreateInfo::builder()
            .layout(pipeline_layout)
            .render_pass(*render_pass)
            .rasterization_state(&rasterization_state_create_info)
            .viewport_state(&viewport_state_create_info)
            .multisample_state(&multisample_state_create_info)
            .input_assembly_state(&input_assembly_state_create_info)
            .vertex_input_state(&vertex_input_state_create_info)
            .color_blend_state(&blend_state_create_info)
            .stages(stages)
            .build();
        let infos = &[graphics_pipeline_create_info];

        let graphics_pipeline = unsafe {
            device
                .handle()
                .create_graphics_pipelines(cache, infos, None)
                .expect("Pipline creation failed")[0]
        };

        let pool_sizes = [ash::vk::DescriptorPoolSize::builder()
            .descriptor_count(1)
            .ty(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .build()];

        let descriptor_pool_create_info = ash::vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(frame_count as u32)
            .build();

        let descriptor_pool = unsafe {
            device
                .handle()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Descriptor pool creations failed")
        };

        let descriptor_set_allocate_info = ash::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(layouts)
            .build();
        let descriptor_sets = unsafe {
            device
                .handle()
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("descriptor set allocation failed")
        };

        let sampler_info = ash::vk::SamplerCreateInfo::builder()
            .min_filter(ash::vk::Filter::LINEAR)
            .mag_filter(ash::vk::Filter::LINEAR)
            .address_mode_u(ash::vk::SamplerAddressMode::REPEAT)
            .address_mode_v(ash::vk::SamplerAddressMode::REPEAT)
            .build();

        let sampler = unsafe {
            device
                .handle()
                .create_sampler(&sampler_info, None)
                .expect("Sampler creation failed")
        };

        Self {
            device: device.handle().clone(),
            descriptor_pool,
            descriptor_sets,
            vertex_shader_module,
            fragment_shader_module,
            pipeline_layout,
            graphics_pipeline,
            sampler,
            to_shader_read_barriers: Vec::new(),
        }
    }

    pub fn render(
        &self,
        command_buffer: &mut CommandBuffer,
        image_view: &ash::vk::ImageView,
        slot: usize,
    ) {
        let image_info = *ash::vk::DescriptorImageInfo::builder()
            .sampler(self.sampler)
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(*image_view);
        let write = *ash::vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_sets[slot])
            .image_info(&[image_info])
            .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER);

        unsafe { self.device.update_descriptor_sets(&[write], &[]) };

        command_buffer.bind_descriptor_sets(
            &self.pipeline_layout,
            ash::vk::PipelineBindPoint::GRAPHICS,
            &[self.descriptor_sets[slot]],
        );

        command_buffer.bind_pipeline(
            ash::vk::PipelineBindPoint::GRAPHICS,
            &self.graphics_pipeline,
        );

        command_buffer.draw_vertices(6, 0, 1, 0);
    }
}

unsafe impl Send for ImageRenderer {}
unsafe impl Sync for ImageRenderer {}
