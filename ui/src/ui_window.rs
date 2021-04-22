use super::application::*;
use super::node::Node;
use super::swapchain;
use super::user_interface::{UIDelegate, UserInterface};
use super::window::MouseEvent;
use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk::Handle;
use skia_safe::gpu::*;
use skia_safe::{Budgeted, Canvas, ImageInfo, Surface};
use std::ffi::c_void;
use std::ptr;

use byteorder::ReadBytesExt;

unsafe fn get_procedure(
    entry: &ash::Entry,
    instance: &ash::Instance,
    of: vk::GetProcOf,
) -> Option<unsafe extern "system" fn()> {
    match of {
        vk::GetProcOf::Instance(instance, name) => {
            let ash_instance = Handle::from_raw(instance as _);
            entry.get_instance_proc_addr(ash_instance, name)
        }

        vk::GetProcOf::Device(device, name) => {
            let ash_device = Handle::from_raw(device as _);
            instance.get_device_proc_addr(ash_device, name)
        }
    }
}

pub trait WindowDelegate<AppState> {
    fn mouse_moved(&mut self, state: &mut AppState, event: &winit::dpi::PhysicalPosition<f64>) {}
    fn mouse_down(&mut self, state: &mut AppState, event: &winit::dpi::PhysicalPosition<f64>) {}
    fn mouse_up(&mut self, state: &mut AppState, event: &winit::dpi::PhysicalPosition<f64>) {}
    fn resized(&mut self, state: &mut AppState, size: &winit::dpi::PhysicalSize<u32>) {}
    fn keyboard_event(&mut self, state: &mut AppState, event: &winit::event::KeyboardInput) {}
    fn draw(&mut self, app: &Application<AppState>, state: &AppState) {}
}

pub struct UIWindow<AppState> {
    context: RecordingContext,
    surfaces: Vec<Surface>,
    surface_images: Vec<ash::vk::Image>,
    surface_image_views: Vec<ash::vk::ImageView>,
    user_interface: UserInterface<AppState>,
    vulkan_surface: ash::vk::SurfaceKHR,
    vulkan_surface_fn: ash::extensions::khr::Surface,

    state: std::marker::PhantomData<AppState>,
    root: Option<Node<AppState>>,
    swapchain: swapchain::Swapchain,
    command_pool: ash::vk::CommandPool,
    command_buffers: Vec<ash::vk::CommandBuffer>,
    semaphores: Vec<ash::vk::Semaphore>,
}

impl<'a, AppState: 'static> UIWindow<AppState> {
    pub fn new(
        app: &'a Application<AppState>,
        state: &AppState,
        window: &winit::window::Window,
        ui_delegate: Box<dyn UIDelegate<AppState>>,
    ) -> Result<Self, &'static str> {
        let (queue, index) = app.present_queue_and_index();

        let entry = app.vulkan_entry();
        let instance = app.vulkan_instance();
        let get_proc = move |of| unsafe {
            if let Some(f) = get_procedure(&entry, &instance, of) {
                f as *const std::ffi::c_void
            } else {
                std::ptr::null()
            }
        };

        let mut context = {
            let backend = unsafe {
                vk::BackendContext::new(
                    app.vulkan_instance().handle().as_raw() as _,
                    app.primary_gpu().as_raw() as _,
                    app.primary_device_context().handle().as_raw() as _,
                    (queue.as_raw() as _, index),
                    &get_proc as _,
                )
            };
            RecordingContext::from(DirectContext::new_vulkan(&backend, None).unwrap())
        };

        let image_info = ImageInfo::new_n32_premul(
            (
                window.inner_size().width as i32,
                window.inner_size().height as i32,
            ),
            None,
        );

        let vulkan_surface = unsafe { ash_window::create_surface(entry, instance, window, None) };
        match vulkan_surface {
            Ok(vs) => {
                let vulkan_surface_fn = ash::extensions::khr::Surface::new(entry, instance);
                let sc = swapchain::Swapchain::new(
                    instance,
                    app.primary_gpu(),
                    app.primary_device_context(),
                    &vulkan_surface_fn,
                    &vs,
                    app.present_queue_and_index().1 as u32,
                    window.inner_size().width,
                    window.inner_size().height,
                );

                let mut surfaces = Vec::new();
                for _ in 0..sc.image_count() {
                    surfaces.push(
                        Surface::new_render_target(
                            &mut context,
                            Budgeted::Yes,
                            &image_info,
                            None,
                            SurfaceOrigin::TopLeft,
                            None,
                            false,
                        )
                        .unwrap(),
                    );
                }

                let surface_images: Vec<ash::vk::Image> = surfaces
                    .iter_mut()
                    .map(|surface| {
                        if let Some(t) = surface
                            .get_backend_texture(skia_safe::surface::BackendHandleAccess::FlushRead)
                        {
                            if let Some(info) = t.vulkan_image_info() {
                                let image: ash::vk::Image =
                                    unsafe { std::mem::transmute(info.image) };
                                return image;
                            }
                        }

                        panic!()
                    })
                    .collect();

                let surface_image_views: Vec<ash::vk::ImageView> = surface_images
                    .iter()
                    .map(|&image| {
                        let create_info = ash::vk::ImageViewCreateInfo::builder()
                            .image(image)
                            .view_type(ash::vk::ImageViewType::TYPE_2D)
                            .format(ash::vk::Format::R8G8B8A8_UNORM)
                            .subresource_range(
                                ash::vk::ImageSubresourceRange::builder()
                                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
                                    .level_count(1)
                                    .layer_count(1)
                                    .build(),
                            )
                            .build();

                        unsafe {
                            app.primary_device_context()
                                .create_image_view(&create_info, None)
                                .expect("ImageView creation failed")
                        }
                    })
                    .collect();

                let pool_create_info = ash::vk::CommandPoolCreateInfo::builder()
                    .flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                    .queue_family_index(app.present_queue_and_index().1 as u32);
                let command_pool = unsafe {
                    app.primary_device_context()
                        .create_command_pool(&pool_create_info, None)
                        .expect("Command pool creation failed for UIWindow")
                };

                let command_buffer_allocate_info = ash::vk::CommandBufferAllocateInfo::builder()
                    .command_buffer_count(sc.image_count() as u32)
                    .command_pool(command_pool)
                    .level(ash::vk::CommandBufferLevel::PRIMARY);

                let command_buffers = unsafe {
                    app.primary_device_context()
                        .allocate_command_buffers(&command_buffer_allocate_info)
                        .expect("Command buffer allocation failed for UIWindow")
                };

                let mut semaphores = Vec::new();
                for _ in 0..sc.image_count() {
                    let semaphore_create_info = ash::vk::SemaphoreCreateInfo::default();

                    semaphores.push(unsafe {
                        app.primary_device_context()
                            .create_semaphore(&semaphore_create_info, None)
                            .unwrap()
                    });
                }

                let mut user_interface = UserInterface::new(ui_delegate.build("root", state));
                user_interface.resize(state, window.inner_size().width, window.inner_size().height);

                let file = std::fs::File::open("shaders/sampled_image.vert.spv")
                    .expect("Shader file not found");
                let meta = std::fs::metadata("shaders/sampled_image.vert.spv")
                    .expect("No metadata found for file");
                let mut buf_reader = std::io::BufReader::new(file);

                let mut buffer = vec![0; (meta.len() / 4) as usize];
                buf_reader
                    .read_u32_into::<byteorder::NativeEndian>(&mut buffer[..])
                    .expect("Failed reading spirv");

                let vertex_shader_module_create_info = ash::vk::ShaderModuleCreateInfo::builder()
                    .code(&buffer)
                    .build();

                let vertex_shader_module = unsafe {
                    app.primary_device_context()
                        .create_shader_module(&vertex_shader_module_create_info, None)
                        .expect("Vertex Shader module creation failed")
                };

                let file = std::fs::File::open("shaders/sampled_image.frag.spv")
                    .expect("Shader file not found");
                let meta = std::fs::metadata("shaders/sampled_image.frag.spv")
                    .expect("No metadata found for file");
                buf_reader = std::io::BufReader::new(file);

                buffer = vec![0; (meta.len() / 4) as usize];
                buf_reader
                    .read_u32_into::<byteorder::NativeEndian>(&mut buffer[..])
                    .expect("Failed reading spirv");

                let fragment_shader_module_create_info = ash::vk::ShaderModuleCreateInfo::builder()
                    .code(&buffer)
                    .build();

                let fragment_shader_module = unsafe {
                    app.primary_device_context()
                        .create_shader_module(&fragment_shader_module_create_info, None)
                        .expect("Vertex Shader module creation failed")
                };

                let vertex_shader_stage_create_info =
                    ash::vk::PipelineShaderStageCreateInfo::builder()
                        .stage(ash::vk::ShaderStageFlags::VERTEX)
                        .module(vertex_shader_module)
                        .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap())
                        .build();

                let fragment_shader_stage_create_info =
                    ash::vk::PipelineShaderStageCreateInfo::builder()
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
                    app.primary_device_context()
                        .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
                        .expect("Failed to create descriptor set layout")
                };

                let layouts = &[descriptor_set_layout];

                let pipeline_layout_create_info = ash::vk::PipelineLayoutCreateInfo::builder()
                    .set_layouts(layouts)
                    .build();

                let pipeline_layout = unsafe {
                    app.primary_device_context()
                        .create_pipeline_layout(&pipeline_layout_create_info, None)
                        .expect("Pipeline layout creation failed")
                };

                let rasterization_state_create_info =
                    ash::vk::PipelineRasterizationStateCreateInfo::builder()
                        .polygon_mode(ash::vk::PolygonMode::FILL)
                        .line_width(1.)
                        .build();

                let viewports = [ash::vk::Viewport::builder()
                    .width(window.inner_size().width as f32)
                    .height(window.inner_size().height as f32)
                    .build()];

                let scissors = [ash::vk::Rect2D::builder()
                    .offset(ash::vk::Offset2D { x: 0, y: 0 })
                    .extent(ash::vk::Extent2D {
                        width: window.inner_size().width,
                        height: window.inner_size().height,
                    })
                    .build()];

                let viewport_state_create_info =
                    ash::vk::PipelineViewportStateCreateInfo::builder()
                        .viewports(&viewports)
                        .scissors(&scissors)
                        .build();

                let multisample_state_create_info =
                    ash::vk::PipelineMultisampleStateCreateInfo::builder()
                        .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1);

                let input_assembly_state_create_info =
                    ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
                        .topology(ash::vk::PrimitiveTopology::TRIANGLE_LIST)
                        .build();

                let vertex_input_state_create_info =
                    ash::vk::PipelineVertexInputStateCreateInfo::builder().build();

                let blend_attachment =
                    [ash::vk::PipelineColorBlendAttachmentState::builder().build()];
                let blend_state_create_info = ash::vk::PipelineColorBlendStateCreateInfo::builder()
                    .attachments(&blend_attachment)
                    .build();

                let pipeline_cache_create_info =
                    ash::vk::PipelineCacheCreateInfo::builder().build();

                let cache = unsafe {
                    app.primary_device_context()
                        .create_pipeline_cache(&pipeline_cache_create_info, None)
                        .expect("Pipeline cache creation failed")
                };

                let graphics_pipeline_create_info = ash::vk::GraphicsPipelineCreateInfo::builder()
                    .layout(pipeline_layout)
                    .render_pass(*sc.render_pass())
                    .rasterization_state(&rasterization_state_create_info)
                    .viewport_state(&viewport_state_create_info)
                    .multisample_state(&multisample_state_create_info)
                    .input_assembly_state(&input_assembly_state_create_info)
                    .vertex_input_state(&vertex_input_state_create_info)
                    .color_blend_state(&blend_state_create_info)
                    .stages(stages)
                    .build();
                let infos = &[graphics_pipeline_create_info];

                let pipeline = unsafe {
                    app.primary_device_context()
                        .create_graphics_pipelines(cache, infos, None)
                        .expect("Pipline creation failed")
                };

                let pool_sizes = [ash::vk::DescriptorPoolSize::builder()
                    .descriptor_count(1)
                    .ty(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .build()];

                let descriptor_pool_create_info = ash::vk::DescriptorPoolCreateInfo::builder()
                    .pool_sizes(&pool_sizes)
                    .max_sets(sc.image_count() as u32)
                    .build();

                let descriptor_pool = unsafe {
                    app.primary_device_context()
                        .create_descriptor_pool(&descriptor_pool_create_info, None)
                        .expect("Descriptor pool creations failed")
                };

                let descriptor_set_allocate_info = ash::vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(layouts);
                let sets = unsafe {
                    app.primary_device_context()
                        .allocate_descriptor_sets(&descriptor_set_allocate_info)
                        .expect("descriptor set allocation failed")
                };

                let sampler_info = ash::vk::SamplerCreateInfo::builder()
                    .min_filter(ash::vk::Filter::LINEAR)
                    .mag_filter(ash::vk::Filter::LINEAR)
                    .address_mode_u(ash::vk::SamplerAddressMode::CLAMP_TO_EDGE)
                    .address_mode_v(ash::vk::SamplerAddressMode::CLAMP_TO_EDGE)
                    .build();

                let sampler = unsafe {
                    app.primary_device_context()
                        .create_sampler(&sampler_info, None)
                        .expect("Sampler creation failed")
                };

                let image_descriptors_infos: Vec<ash::vk::DescriptorImageInfo> =
                    surface_image_views
                        .iter()
                        .map(|&image_view| {
                            ash::vk::DescriptorImageInfo::builder()
                                .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                                .image_view(image_view)
                                .sampler(sampler)
                                .build()
                        })
                        .collect();

                let writes: Vec<ash::vk::WriteDescriptorSet> = image_descriptors_infos
                    .iter()
                    .map(|&info| {
                        let image_info = [info];
                        ash::vk::WriteDescriptorSet::builder()
                            .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .image_info(&image_info)
                            .build()
                    })
                    .collect();

                unsafe {
                    app.primary_device_context()
                        .update_descriptor_sets(&writes, &[])
                };

                Ok(Self {
                    context,
                    surfaces,
                    surface_images,
                    surface_image_views,
                    user_interface,
                    vulkan_surface_fn,
                    vulkan_surface: vs,
                    state: std::marker::PhantomData::<AppState>::default(),
                    root: None,
                    swapchain: sc,
                    command_pool,
                    command_buffers,
                    semaphores,
                })
            }
            Err(_result) => Err("Swapchain creation failed"),
        }
    }
}

impl<'a, AppState: 'static> WindowDelegate<AppState> for UIWindow<AppState> {
    fn mouse_moved(&mut self, state: &mut AppState, event: &winit::dpi::PhysicalPosition<f64>) {
        if let Some(root) = &mut self.root {
            let p = skia_safe::Point::from((event.x as f32, event.y as f32));
            root.mouse_moved(state, &MouseEvent::new(0, &p, &p));
        }
    }

    fn resized(&mut self, _: &mut AppState, event: &winit::dpi::PhysicalSize<u32>) {
        let image_info = ImageInfo::new_n32_premul((event.width as i32, event.height as i32), None);

        for s in 0..self.swapchain.image_count() {
            self.surfaces[s] = Surface::new_render_target(
                &mut self.context,
                Budgeted::Yes,
                &image_info,
                None,
                SurfaceOrigin::TopLeft,
                None,
                false,
            )
            .unwrap()
        }
    }
    fn draw(&mut self, app: &Application<AppState>, state: &AppState) {
        // Next swapchain image
        let (success, image_index, framebuffer) = self.swapchain.next_frame_buffer();

        // draw user interface
        self.user_interface
            .paint(state, self.surfaces[image_index as usize].canvas());
        self.surfaces[image_index as usize].flush_and_submit();

        // get the texture from skia back to transition into sampled image
        if let Some(t) = self.surfaces[image_index as usize]
            .get_backend_texture(skia_safe::surface::BackendHandleAccess::FlushRead)
        {
            if let Some(info) = t.vulkan_image_info() {
                let barrier = ash::vk::ImageMemoryBarrier::builder()
                    .old_layout(ash::vk::ImageLayout::UNDEFINED)
                    .new_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image(unsafe { std::mem::transmute(info.image) })
                    .src_queue_family_index(info.current_queue_family)
                    .dst_queue_family_index(info.current_queue_family)
                    .subresource_range(
                        ash::vk::ImageSubresourceRange::builder()
                            .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
                            .layer_count(1)
                            .level_count(1)
                            .build(),
                    )
                    .build();
                let device = app.primary_device_context();
                let commands = &self.command_buffers[image_index as usize];
                let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::builder().build();
                unsafe {
                    device
                        .begin_command_buffer(*commands, &command_buffer_begin_info)
                        .expect("Unable to start recording command buffer");

                    device.cmd_pipeline_barrier(
                        *commands,
                        ash::vk::PipelineStageFlags::ALL_COMMANDS,
                        ash::vk::PipelineStageFlags::ALL_COMMANDS,
                        ash::vk::DependencyFlags::BY_REGION,
                        &[],
                        &[],
                        &[barrier],
                    );

                    let clear_values = [ash::vk::ClearValue {
                        color: ash::vk::ClearColorValue {
                            float32: [0.0, 1.0, 0.0, 1.0],
                        },
                    }];

                    // Begin renderpass to transition swapchain image into color attachment and output as Present Source
                    let render_pass_begin_info = ash::vk::RenderPassBeginInfo::builder()
                        .render_pass(*self.swapchain.render_pass())
                        .framebuffer(*framebuffer)
                        .clear_values(&clear_values)
                        .render_area(ash::vk::Rect2D {
                            offset: ash::vk::Offset2D { x: 0, y: 0 },
                            extent: ash::vk::Extent2D {
                                width: self.surfaces[0].width() as u32,
                                height: self.surfaces[0].height() as u32,
                            },
                        });
                    device.cmd_begin_render_pass(
                        *commands,
                        &render_pass_begin_info,
                        ash::vk::SubpassContents::INLINE,
                    );
                    device.cmd_end_render_pass(*commands);
                    device
                        .end_command_buffer(*commands)
                        .expect("End recording command buffer failed");

                    let buffers = &[*commands];
                    let submit_info = ash::vk::SubmitInfo::builder()
                        .command_buffers(buffers)
                        .signal_semaphores(&[self.semaphores[image_index as usize]])
                        .wait_dst_stage_mask(&[ash::vk::PipelineStageFlags::ALL_GRAPHICS])
                        .wait_semaphores(&[])
                        .build();
                    device
                        .queue_submit(
                            *app.present_queue_and_index().0,
                            &[submit_info],
                            ash::vk::Fence::null(),
                        )
                        .expect("queue submit failed.");
                }
            }
        }

        self.swapchain.swap(
            app.present_queue_and_index().0,
            &self.semaphores[image_index as usize],
            image_index,
        )
    }
}
