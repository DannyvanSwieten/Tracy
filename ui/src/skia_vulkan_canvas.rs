use crate::canvas_2d::Canvas2D;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk::Handle;
use skia_safe::gpu::{vk, DirectContext, RecordingContext, SemaphoresSubmitted, SurfaceOrigin};
use skia_safe::{Budgeted, Color, Font, Image, ImageInfo, Paint, Point, Rect, Surface};
use vk_utils::device_context::DeviceContext;

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

pub struct SkiaGpuCanvas2D {
    context: RecordingContext,
    surfaces: Vec<Surface>,
    surface_images: Vec<ash::vk::Image>,
    surface_image_views: Vec<ash::vk::ImageView>,
    current_image_index: usize,
}

impl SkiaGpuCanvas2D {
    pub fn new(device: &DeviceContext, image_count: usize, width: u32, height: u32) -> Self {
        let queue = device.graphics_queue().unwrap();

        let entry = device.gpu().vulkan().library();
        let instance = device.gpu().vulkan().vk_instance();
        let get_proc = move |of| unsafe {
            if let Some(f) = get_procedure(&entry, instance, of) {
                f as *const std::ffi::c_void
            } else {
                std::ptr::null()
            }
        };

        let mut context = {
            let backend = unsafe {
                vk::BackendContext::new(
                    instance.handle().as_raw() as _,
                    device.gpu().vk_physical_device().as_raw() as _,
                    device.vk_device().handle().as_raw() as _,
                    (
                        queue.vk_queue().as_raw() as _,
                        queue.family_type_index() as usize,
                    ),
                    &get_proc as _,
                )
            };
            RecordingContext::from(DirectContext::new_vulkan(&backend, None).unwrap())
        };

        let image_info = ImageInfo::new_n32_premul((width as i32, height as i32), None);

        let mut surfaces = Vec::new();
        for _ in 0..image_count {
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
                if let Some(t) =
                    surface.get_backend_texture(skia_safe::surface::BackendHandleAccess::FlushRead)
                {
                    if let Some(info) = t.vulkan_image_info() {
                        let image: ash::vk::Image = unsafe { std::mem::transmute(info.image) };
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
                    .format(ash::vk::Format::B8G8R8A8_UNORM)
                    .subresource_range(
                        ash::vk::ImageSubresourceRange::builder()
                            .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
                            .level_count(1)
                            .layer_count(1)
                            .build(),
                    )
                    .build();

                unsafe {
                    device
                        .vk_device()
                        .create_image_view(&create_info, None)
                        .expect("ImageView creation failed")
                }
            })
            .collect();

        let semaphores: Vec<SemaphoresSubmitted> =
            (0..image_count).map(|_| SemaphoresSubmitted::No).collect();

        Self {
            context,
            surfaces,
            surface_images,
            surface_image_views,
            current_image_index: 0,
        }
    }
}

impl Canvas2D for SkiaGpuCanvas2D {
    fn clear(&mut self, color: &Color) {
        self.surfaces[self.current_image_index]
            .canvas()
            .clear(*color);
    }

    fn save(&mut self) {
        self.surfaces[self.current_image_index].canvas().save();
    }

    fn restore(&mut self) {
        self.surfaces[self.current_image_index].canvas().restore();
    }

    fn draw_rect(&mut self, rect: &Rect, paint: &Paint) {
        self.surfaces[self.current_image_index]
            .canvas()
            .draw_rect(rect, paint);
    }
    fn draw_rounded_rect(&mut self, rect: &Rect, rx: f32, ry: f32, paint: &Paint) {
        self.surfaces[self.current_image_index]
            .canvas()
            .draw_round_rect(rect, rx, ry, paint);
    }

    fn draw_string(&mut self, text: &str, center: &Point, font: &Font, paint: &Paint) {
        let blob = skia_safe::TextBlob::from_str(text.to_string(), font);
        if let Some(b) = blob {
            let rect = b.bounds();
            let left = *center - rect.center();
            self.surfaces[self.current_image_index]
                .canvas()
                .draw_str(text, left, font, paint);
        }
    }

    fn draw_vk_image(&mut self, image: &ash::vk::Image, width: u32, height: u32) {
        let sk_vk_image: vk::Image = unsafe { std::mem::transmute(*image) };
        let info = unsafe {
            vk::ImageInfo::new(
                sk_vk_image,
                vk::Alloc::default(),
                vk::ImageTiling::OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::Format::R8G8B8A8_UNORM,
                1,
                0,
                None,
                None,
                None,
            )
        };

        let backend_texture = unsafe {
            skia_safe::gpu::BackendTexture::new_vulkan((width as i32, height as i32), &info)
        };

        let sk_image = Image::from_texture(
            &mut self.context,
            &backend_texture,
            skia_safe::gpu::SurfaceOrigin::TopLeft,
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            skia_safe::ColorSpace::new_srgb_linear(),
        );

        if let Some(image) = sk_image {
            self.surfaces[self.current_image_index]
                .canvas()
                .draw_image(image, (0., 0.), None);
        }
    }

    fn draw_vk_image_rect(&mut self, src_rect: &Rect, dst_rect: &Rect, image: &ash::vk::Image) {
        let sk_vk_image: vk::Image = unsafe { std::mem::transmute(*image) };
        let info = unsafe {
            vk::ImageInfo::new(
                sk_vk_image,
                vk::Alloc::default(),
                vk::ImageTiling::OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::Format::R8G8B8A8_UNORM,
                1,
                0,
                None,
                None,
                None,
            )
        };

        let backend_texture = unsafe {
            skia_safe::gpu::BackendTexture::new_vulkan(
                (src_rect.width() as i32, src_rect.height() as i32),
                &info,
            )
        };

        let sk_image = Image::from_texture(
            &mut self.context,
            &backend_texture,
            skia_safe::gpu::SurfaceOrigin::TopLeft,
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            skia_safe::ColorSpace::new_srgb_linear(),
        );

        let constraint = skia_safe::canvas::SrcRectConstraint::Fast;

        if let Some(image) = sk_image {
            self.surfaces[self.current_image_index]
                .canvas()
                .draw_image_rect(
                    image,
                    Some((src_rect, constraint)),
                    dst_rect,
                    &Paint::default(),
                );
        }
    }

    fn flush(&mut self) -> (ash::vk::Image, ash::vk::ImageView) {
        if let Some(direct) = self.context.as_direct_context().as_mut() {
            direct.flush_submit_and_sync_cpu();
        }

        let view = self.surface_image_views[self.current_image_index];
        let image = self.surface_images[self.current_image_index];
        self.current_image_index += 1;
        self.current_image_index %= self.surface_images.len();

        (image, view)
    }
}
