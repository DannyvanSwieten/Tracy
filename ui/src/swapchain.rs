use ash::version::DeviceV1_0;
use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk::Handle;

pub struct Swapchain<'a> {
    queue: &'a ash::vk::Queue,
    handle: ash::vk::SwapchainKHR,
    images: Vec<ash::vk::Image>,
    image_views: Vec<ash::vk::ImageView>,
    present_semaphore: ash::vk::Semaphore,
}

impl<'a> Swapchain<'a> {
    pub fn new(
        instance: &ash::Instance,
        gpu: &ash::vk::PhysicalDevice,
        ctx: &ash::Device,
        queue: &'a ash::vk::Queue,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &ash::vk::SurfaceKHR,
        queue_index: u32,
        width: u32,
        height: u32,
    ) -> Self {
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
            .image_array_layers(1);

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

        let semaphore_create_info = ash::vk::SemaphoreCreateInfo::default();

        let present_semaphore =
            unsafe { ctx.create_semaphore(&semaphore_create_info, None).unwrap() };

        Self {
            handle: swapchain,
            images,
            image_views,
            present_semaphore,
            queue: &queue,
        }
    }
}
