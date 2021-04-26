use ash::version::DeviceV1_0;

pub struct Swapchain {
    swapchain_loader: ash::extensions::khr::Swapchain,
    handle: ash::vk::SwapchainKHR,
    images: Vec<ash::vk::Image>,
    image_views: Vec<ash::vk::ImageView>,
    present_semaphores: Vec<ash::vk::Semaphore>,
    renderpass: ash::vk::RenderPass,
    framebuffers: Vec<ash::vk::Framebuffer>,
    current_index: u32,
    format: ash::vk::Format,

    width: u32,
    height: u32,
}

fn create_swapchain(
    instance: &ash::Instance,
    gpu: &ash::vk::PhysicalDevice,
    ctx: &ash::Device,
    surface_loader: &ash::extensions::khr::Surface,
    surface: &ash::vk::SurfaceKHR,
    swapchain_loader: &ash::extensions::khr::Swapchain,
    old_swapchain: Option<&ash::vk::SwapchainKHR>,
    queue_index: u32,
    width: u32,
    height: u32,
) -> (
    ash::vk::SwapchainKHR,
    Vec<ash::vk::Image>,
    Vec<ash::vk::ImageView>,
    ash::vk::SurfaceFormatKHR,
) {
    let _ = unsafe {
        surface_loader
            .get_physical_device_surface_support(*gpu, queue_index, *surface)
            .expect("Query physical device queue surface support failed")
    };

    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(*gpu, *surface)
            .expect("No surface formats found for surface / device combination")
    };

    // Choose first format for now.
    let format = formats[0];
    let capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(*gpu, *surface)
            .expect("No surface capabilities found for surface / device combination")
    };
    let mut desired_image_count = capabilities.min_image_count + 1;
    if capabilities.max_image_count > 0 && desired_image_count > capabilities.max_image_count {
        desired_image_count = capabilities.max_image_count;
    }
    let surface_resolution = match capabilities.current_extent.width {
        std::u32::MAX => ash::vk::Extent2D { width, height },
        _ => capabilities.current_extent,
    };
    let pre_transform = if capabilities
        .supported_transforms
        .contains(ash::vk::SurfaceTransformFlagsKHR::IDENTITY)
    {
        ash::vk::SurfaceTransformFlagsKHR::IDENTITY
    } else {
        capabilities.current_transform
    };
    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(*gpu, *surface)
            .expect("No present modes found")
    };
    let present_mode = present_modes
        .iter()
        .cloned()
        .find(|&mode| mode == ash::vk::PresentModeKHR::MAILBOX)
        .unwrap_or(ash::vk::PresentModeKHR::FIFO);
    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, ctx);
    let mut old = ash::vk::SwapchainKHR::null();
    if old_swapchain.is_some() {
        old = *old_swapchain.unwrap();
    }
    let swapchain_create_info = ash::vk::SwapchainCreateInfoKHR::builder()
        .surface(*surface)
        .min_image_count(desired_image_count)
        .image_color_space(format.color_space)
        .image_format(format.format)
        .image_extent(surface_resolution)
        .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
        .pre_transform(pre_transform)
        .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .image_array_layers(1)
        .old_swapchain(old);

    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Swapchain creation failed")
    };

    let images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .expect("Acquire swapchain images failed")
    };

    let image_views: Vec<ash::vk::ImageView> = images
        .iter()
        .map(|&image| {
            let create_view_info = ash::vk::ImageViewCreateInfo::builder()
                .view_type(ash::vk::ImageViewType::TYPE_2D)
                .format(format.format)
                .components(ash::vk::ComponentMapping {
                    r: ash::vk::ComponentSwizzle::R,
                    g: ash::vk::ComponentSwizzle::G,
                    b: ash::vk::ComponentSwizzle::B,
                    a: ash::vk::ComponentSwizzle::A,
                })
                .subresource_range(ash::vk::ImageSubresourceRange {
                    aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(image);
            unsafe {
                ctx.create_image_view(&create_view_info, None)
                    .expect("Image view creation for swapchain images failed")
            }
        })
        .collect();

    (swapchain, images, image_views, format)
}

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        gpu: &ash::vk::PhysicalDevice,
        ctx: &ash::Device,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &ash::vk::SurfaceKHR,
        swapchain_loader: &ash::extensions::khr::Swapchain,
        old_swapchain: Option<&ash::vk::SwapchainKHR>,
        queue_index: u32,
        width: u32,
        height: u32,
    ) -> Self {
        let (swapchain, images, image_views, format) = create_swapchain(
            instance,
            gpu,
            ctx,
            surface_loader,
            surface,
            swapchain_loader,
            old_swapchain,
            queue_index,
            width,
            height,
        );

        let attachments = [ash::vk::AttachmentDescription {
            format: format.format,
            samples: ash::vk::SampleCountFlags::TYPE_1,
            load_op: ash::vk::AttachmentLoadOp::DONT_CARE,
            store_op: ash::vk::AttachmentStoreOp::STORE,
            final_layout: ash::vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        }];

        let attachment_refs = [ash::vk::AttachmentReference {
            attachment: 0,
            layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let dependencies = [ash::vk::SubpassDependency {
            src_subpass: ash::vk::SUBPASS_EXTERNAL,
            src_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_READ
                | ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpasses = [ash::vk::SubpassDescription::builder()
            .color_attachments(&attachment_refs)
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
            .build()];

        let renderpass_create_info = ash::vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let renderpass = unsafe {
            ctx.create_render_pass(&renderpass_create_info, None)
                .expect("Renderpass creation failed for swapchain")
        };

        let framebuffers: Vec<ash::vk::Framebuffer> = image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view];
                let create_info = ash::vk::FramebufferCreateInfo::builder()
                    .render_pass(renderpass)
                    .attachments(&attachments)
                    .width(width)
                    .height(height)
                    .layers(1);
                unsafe {
                    ctx.create_framebuffer(&create_info, None)
                        .expect("Framebuffer creation failed for swapchain images")
                }
            })
            .collect();

        let semaphore_create_info = ash::vk::SemaphoreCreateInfo::default();

        let mut present_semaphores = Vec::new();
        for _ in 0..images.len() {
            present_semaphores
                .push(unsafe { ctx.create_semaphore(&semaphore_create_info, None).unwrap() });
        }

        Self {
            handle: swapchain,
            swapchain_loader: ash::extensions::khr::Swapchain::new(instance, ctx),
            images,
            image_views,
            present_semaphores,
            renderpass,
            framebuffers,
            current_index: 0,
            format: format.format,
            width,
            height,
        }
    }

    pub fn next_frame_buffer(
        &mut self,
    ) -> Result<(bool, u32, ash::vk::Framebuffer, ash::vk::Semaphore), ash::vk::Result> {
        unsafe {
            let result = self.swapchain_loader.acquire_next_image(
                self.handle,
                std::u64::MAX,
                self.present_semaphores[self.current_index as usize],
                ash::vk::Fence::null(),
            );

            match result {
                Ok((index, sub_optimal)) => {
                    let result = (
                        true,
                        index,
                        self.framebuffers[index as usize],
                        self.present_semaphores[index as usize],
                    );
                    self.current_index = self.current_index + 1;
                    self.current_index = self.current_index % self.image_count() as u32;
                    return Ok(result);
                }

                Err(code) => return Err(code),
            }
        };
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn render_pass(&self) -> &ash::vk::RenderPass {
        &self.renderpass
    }

    pub fn semaphore(&self, index: usize) -> &ash::vk::Semaphore {
        &self.present_semaphores[index]
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    pub fn images(&self) -> &Vec<ash::vk::Image> {
        &self.images
    }

    pub fn format(&self) -> &ash::vk::Format {
        &self.format
    }

    pub fn handle(&self) -> &ash::vk::SwapchainKHR {
        &self.handle
    }

    pub fn swap(&self, queue: &ash::vk::Queue, semaphore: &ash::vk::Semaphore, index: u32) -> bool {
        let s = &[*semaphore];
        let sc = &[self.handle];
        let i = &[index];
        let present_info = ash::vk::PresentInfoKHR::builder()
            .wait_semaphores(s)
            .swapchains(sc)
            .image_indices(i);

        unsafe {
            let r = self.swapchain_loader.queue_present(*queue, &present_info);

            if r.is_err() {
                true
            } else {
                false
            }
        }
    }
}
