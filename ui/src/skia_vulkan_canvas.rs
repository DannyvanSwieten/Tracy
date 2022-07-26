use std::cell::RefCell;
use std::rc::Rc;

use crate::canvas_2d::Canvas2D;
use ash::vk::Handle;
use skia_safe::gpu::{vk, DirectContext, RecordingContext, SemaphoresSubmitted, SurfaceOrigin};
use skia_safe::{Budgeted, Color, Font, Image, ImageInfo, Paint, Point, Rect, Surface};
use vk_utils::device_context::DeviceContext;
use vk_utils::image_resource::ImageResource;
use vk_utils::queue::CommandQueue;

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

pub struct SkiaCanvasImage {
    skia_backend_texture: skia_safe::gpu::BackendTexture,
    width: u32,
    height: u32,
}

impl SkiaCanvasImage {
    pub fn new(
        skia_backend_texture: skia_safe::gpu::BackendTexture,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            skia_backend_texture,
            width,
            height,
        }
    }
}

impl ImageResource for SkiaCanvasImage {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn depth(&self) -> u32 {
        1
    }

    fn format(&self) -> ash::vk::Format {
        if let Some(info) = self.skia_backend_texture.vulkan_image_info() {
            match info.format {
                vk::Format::UNDEFINED => todo!(),
                vk::Format::R4G4_UNORM_PACK8 => todo!(),
                vk::Format::R4G4B4A4_UNORM_PACK16 => todo!(),
                vk::Format::B4G4R4A4_UNORM_PACK16 => todo!(),
                vk::Format::R5G6B5_UNORM_PACK16 => todo!(),
                vk::Format::B5G6R5_UNORM_PACK16 => todo!(),
                vk::Format::R5G5B5A1_UNORM_PACK16 => todo!(),
                vk::Format::B5G5R5A1_UNORM_PACK16 => todo!(),
                vk::Format::A1R5G5B5_UNORM_PACK16 => todo!(),
                vk::Format::R8_UNORM => todo!(),
                vk::Format::R8_SNORM => todo!(),
                vk::Format::R8_USCALED => todo!(),
                vk::Format::R8_SSCALED => todo!(),
                vk::Format::R8_UINT => todo!(),
                vk::Format::R8_SINT => todo!(),
                vk::Format::R8_SRGB => todo!(),
                vk::Format::R8G8_UNORM => todo!(),
                vk::Format::R8G8_SNORM => todo!(),
                vk::Format::R8G8_USCALED => todo!(),
                vk::Format::R8G8_SSCALED => todo!(),
                vk::Format::R8G8_UINT => todo!(),
                vk::Format::R8G8_SINT => todo!(),
                vk::Format::R8G8_SRGB => todo!(),
                vk::Format::R8G8B8_UNORM => todo!(),
                vk::Format::R8G8B8_SNORM => todo!(),
                vk::Format::R8G8B8_USCALED => todo!(),
                vk::Format::R8G8B8_SSCALED => todo!(),
                vk::Format::R8G8B8_UINT => todo!(),
                vk::Format::R8G8B8_SINT => todo!(),
                vk::Format::R8G8B8_SRGB => todo!(),
                vk::Format::B8G8R8_UNORM => todo!(),
                vk::Format::B8G8R8_SNORM => todo!(),
                vk::Format::B8G8R8_USCALED => todo!(),
                vk::Format::B8G8R8_SSCALED => todo!(),
                vk::Format::B8G8R8_UINT => todo!(),
                vk::Format::B8G8R8_SINT => todo!(),
                vk::Format::B8G8R8_SRGB => todo!(),
                vk::Format::R8G8B8A8_UNORM => ash::vk::Format::R8G8B8A8_UNORM,
                vk::Format::R8G8B8A8_SNORM => ash::vk::Format::R8G8B8A8_SNORM,
                vk::Format::R8G8B8A8_USCALED => todo!(),
                vk::Format::R8G8B8A8_SSCALED => todo!(),
                vk::Format::R8G8B8A8_UINT => ash::vk::Format::R8G8B8A8_UINT,
                vk::Format::R8G8B8A8_SINT => ash::vk::Format::R8G8B8A8_SINT,
                vk::Format::R8G8B8A8_SRGB => todo!(),
                vk::Format::B8G8R8A8_UNORM => todo!(),
                vk::Format::B8G8R8A8_SNORM => todo!(),
                vk::Format::B8G8R8A8_USCALED => todo!(),
                vk::Format::B8G8R8A8_SSCALED => todo!(),
                vk::Format::B8G8R8A8_UINT => todo!(),
                vk::Format::B8G8R8A8_SINT => todo!(),
                vk::Format::B8G8R8A8_SRGB => todo!(),
                vk::Format::A8B8G8R8_UNORM_PACK32 => todo!(),
                vk::Format::A8B8G8R8_SNORM_PACK32 => todo!(),
                vk::Format::A8B8G8R8_USCALED_PACK32 => todo!(),
                vk::Format::A8B8G8R8_SSCALED_PACK32 => todo!(),
                vk::Format::A8B8G8R8_UINT_PACK32 => todo!(),
                vk::Format::A8B8G8R8_SINT_PACK32 => todo!(),
                vk::Format::A8B8G8R8_SRGB_PACK32 => todo!(),
                vk::Format::A2R10G10B10_UNORM_PACK32 => todo!(),
                vk::Format::A2R10G10B10_SNORM_PACK32 => todo!(),
                vk::Format::A2R10G10B10_USCALED_PACK32 => todo!(),
                vk::Format::A2R10G10B10_SSCALED_PACK32 => todo!(),
                vk::Format::A2R10G10B10_UINT_PACK32 => todo!(),
                vk::Format::A2R10G10B10_SINT_PACK32 => todo!(),
                vk::Format::A2B10G10R10_UNORM_PACK32 => todo!(),
                vk::Format::A2B10G10R10_SNORM_PACK32 => todo!(),
                vk::Format::A2B10G10R10_USCALED_PACK32 => todo!(),
                vk::Format::A2B10G10R10_SSCALED_PACK32 => todo!(),
                vk::Format::A2B10G10R10_UINT_PACK32 => todo!(),
                vk::Format::A2B10G10R10_SINT_PACK32 => todo!(),
                vk::Format::R16_UNORM => todo!(),
                vk::Format::R16_SNORM => todo!(),
                vk::Format::R16_USCALED => todo!(),
                vk::Format::R16_SSCALED => todo!(),
                vk::Format::R16_UINT => todo!(),
                vk::Format::R16_SINT => todo!(),
                vk::Format::R16_SFLOAT => todo!(),
                vk::Format::R16G16_UNORM => todo!(),
                vk::Format::R16G16_SNORM => todo!(),
                vk::Format::R16G16_USCALED => todo!(),
                vk::Format::R16G16_SSCALED => todo!(),
                vk::Format::R16G16_UINT => todo!(),
                vk::Format::R16G16_SINT => todo!(),
                vk::Format::R16G16_SFLOAT => todo!(),
                vk::Format::R16G16B16_UNORM => todo!(),
                vk::Format::R16G16B16_SNORM => todo!(),
                vk::Format::R16G16B16_USCALED => todo!(),
                vk::Format::R16G16B16_SSCALED => todo!(),
                vk::Format::R16G16B16_UINT => todo!(),
                vk::Format::R16G16B16_SINT => todo!(),
                vk::Format::R16G16B16_SFLOAT => todo!(),
                vk::Format::R16G16B16A16_UNORM => todo!(),
                vk::Format::R16G16B16A16_SNORM => todo!(),
                vk::Format::R16G16B16A16_USCALED => todo!(),
                vk::Format::R16G16B16A16_SSCALED => todo!(),
                vk::Format::R16G16B16A16_UINT => todo!(),
                vk::Format::R16G16B16A16_SINT => todo!(),
                vk::Format::R16G16B16A16_SFLOAT => todo!(),
                vk::Format::R32_UINT => todo!(),
                vk::Format::R32_SINT => todo!(),
                vk::Format::R32_SFLOAT => todo!(),
                vk::Format::R32G32_UINT => todo!(),
                vk::Format::R32G32_SINT => todo!(),
                vk::Format::R32G32_SFLOAT => todo!(),
                vk::Format::R32G32B32_UINT => todo!(),
                vk::Format::R32G32B32_SINT => todo!(),
                vk::Format::R32G32B32_SFLOAT => todo!(),
                vk::Format::R32G32B32A32_UINT => todo!(),
                vk::Format::R32G32B32A32_SINT => todo!(),
                vk::Format::R32G32B32A32_SFLOAT => todo!(),
                vk::Format::R64_UINT => todo!(),
                vk::Format::R64_SINT => todo!(),
                vk::Format::R64_SFLOAT => todo!(),
                vk::Format::R64G64_UINT => todo!(),
                vk::Format::R64G64_SINT => todo!(),
                vk::Format::R64G64_SFLOAT => todo!(),
                vk::Format::R64G64B64_UINT => todo!(),
                vk::Format::R64G64B64_SINT => todo!(),
                vk::Format::R64G64B64_SFLOAT => todo!(),
                vk::Format::R64G64B64A64_UINT => todo!(),
                vk::Format::R64G64B64A64_SINT => todo!(),
                vk::Format::R64G64B64A64_SFLOAT => todo!(),
                vk::Format::B10G11R11_UFLOAT_PACK32 => todo!(),
                vk::Format::E5B9G9R9_UFLOAT_PACK32 => todo!(),
                vk::Format::D16_UNORM => todo!(),
                vk::Format::X8_D24_UNORM_PACK32 => todo!(),
                vk::Format::D32_SFLOAT => todo!(),
                vk::Format::S8_UINT => todo!(),
                vk::Format::D16_UNORM_S8_UINT => todo!(),
                vk::Format::D24_UNORM_S8_UINT => todo!(),
                vk::Format::D32_SFLOAT_S8_UINT => todo!(),
                vk::Format::BC1_RGB_UNORM_BLOCK => todo!(),
                vk::Format::BC1_RGB_SRGB_BLOCK => todo!(),
                vk::Format::BC1_RGBA_UNORM_BLOCK => todo!(),
                vk::Format::BC1_RGBA_SRGB_BLOCK => todo!(),
                vk::Format::BC2_UNORM_BLOCK => todo!(),
                vk::Format::BC2_SRGB_BLOCK => todo!(),
                vk::Format::BC3_UNORM_BLOCK => todo!(),
                vk::Format::BC3_SRGB_BLOCK => todo!(),
                vk::Format::BC4_UNORM_BLOCK => todo!(),
                vk::Format::BC4_SNORM_BLOCK => todo!(),
                vk::Format::BC5_UNORM_BLOCK => todo!(),
                vk::Format::BC5_SNORM_BLOCK => todo!(),
                vk::Format::BC6H_UFLOAT_BLOCK => todo!(),
                vk::Format::BC6H_SFLOAT_BLOCK => todo!(),
                vk::Format::BC7_UNORM_BLOCK => todo!(),
                vk::Format::BC7_SRGB_BLOCK => todo!(),
                vk::Format::ETC2_R8G8B8_UNORM_BLOCK => todo!(),
                vk::Format::ETC2_R8G8B8_SRGB_BLOCK => todo!(),
                vk::Format::ETC2_R8G8B8A1_UNORM_BLOCK => todo!(),
                vk::Format::ETC2_R8G8B8A1_SRGB_BLOCK => todo!(),
                vk::Format::ETC2_R8G8B8A8_UNORM_BLOCK => todo!(),
                vk::Format::ETC2_R8G8B8A8_SRGB_BLOCK => todo!(),
                vk::Format::EAC_R11_UNORM_BLOCK => todo!(),
                vk::Format::EAC_R11_SNORM_BLOCK => todo!(),
                vk::Format::EAC_R11G11_UNORM_BLOCK => todo!(),
                vk::Format::EAC_R11G11_SNORM_BLOCK => todo!(),
                vk::Format::ASTC_4x4_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_4x4_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_5x4_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_5x4_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_5x5_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_5x5_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_6x5_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_6x5_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_6x6_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_6x6_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_8x5_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_8x5_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_8x6_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_8x6_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_8x8_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_8x8_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_10x5_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_10x5_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_10x6_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_10x6_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_10x8_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_10x8_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_10x10_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_10x10_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_12x10_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_12x10_SRGB_BLOCK => todo!(),
                vk::Format::ASTC_12x12_UNORM_BLOCK => todo!(),
                vk::Format::ASTC_12x12_SRGB_BLOCK => todo!(),
                vk::Format::G8B8G8R8_422_UNORM => todo!(),
                vk::Format::B8G8R8G8_422_UNORM => todo!(),
                vk::Format::G8_B8_R8_3PLANE_420_UNORM => todo!(),
                vk::Format::G8_B8R8_2PLANE_420_UNORM => todo!(),
                vk::Format::G8_B8_R8_3PLANE_422_UNORM => todo!(),
                vk::Format::G8_B8R8_2PLANE_422_UNORM => todo!(),
                vk::Format::G8_B8_R8_3PLANE_444_UNORM => todo!(),
                vk::Format::R10X6_UNORM_PACK16 => todo!(),
                vk::Format::R10X6G10X6_UNORM_2PACK16 => todo!(),
                vk::Format::R10X6G10X6B10X6A10X6_UNORM_4PACK16 => todo!(),
                vk::Format::G10X6B10X6G10X6R10X6_422_UNORM_4PACK16 => todo!(),
                vk::Format::B10X6G10X6R10X6G10X6_422_UNORM_4PACK16 => todo!(),
                vk::Format::G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16 => todo!(),
                vk::Format::G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16 => todo!(),
                vk::Format::G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16 => todo!(),
                vk::Format::G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16 => todo!(),
                vk::Format::G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16 => todo!(),
                vk::Format::R12X4_UNORM_PACK16 => todo!(),
                vk::Format::R12X4G12X4_UNORM_2PACK16 => todo!(),
                vk::Format::R12X4G12X4B12X4A12X4_UNORM_4PACK16 => todo!(),
                vk::Format::G12X4B12X4G12X4R12X4_422_UNORM_4PACK16 => todo!(),
                vk::Format::B12X4G12X4R12X4G12X4_422_UNORM_4PACK16 => todo!(),
                vk::Format::G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16 => todo!(),
                vk::Format::G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16 => todo!(),
                vk::Format::G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16 => todo!(),
                vk::Format::G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16 => todo!(),
                vk::Format::G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16 => todo!(),
                vk::Format::G16B16G16R16_422_UNORM => todo!(),
                vk::Format::B16G16R16G16_422_UNORM => todo!(),
                vk::Format::G16_B16_R16_3PLANE_420_UNORM => todo!(),
                vk::Format::G16_B16R16_2PLANE_420_UNORM => todo!(),
                vk::Format::G16_B16_R16_3PLANE_422_UNORM => todo!(),
                vk::Format::G16_B16R16_2PLANE_422_UNORM => todo!(),
                vk::Format::G16_B16_R16_3PLANE_444_UNORM => todo!(),
                vk::Format::PVRTC1_2BPP_UNORM_BLOCK_IMG => todo!(),
                vk::Format::PVRTC1_4BPP_UNORM_BLOCK_IMG => todo!(),
                vk::Format::PVRTC2_2BPP_UNORM_BLOCK_IMG => todo!(),
                vk::Format::PVRTC2_4BPP_UNORM_BLOCK_IMG => todo!(),
                vk::Format::PVRTC1_2BPP_SRGB_BLOCK_IMG => todo!(),
                vk::Format::PVRTC1_4BPP_SRGB_BLOCK_IMG => todo!(),
                vk::Format::PVRTC2_2BPP_SRGB_BLOCK_IMG => todo!(),
                vk::Format::PVRTC2_4BPP_SRGB_BLOCK_IMG => todo!(),
                vk::Format::ASTC_4x4_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_5x4_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_5x5_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_6x5_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_6x6_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_8x5_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_8x6_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_8x8_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_10x5_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_10x6_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_10x8_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_10x10_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_12x10_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::ASTC_12x12_SFLOAT_BLOCK_EXT => todo!(),
                vk::Format::G8_B8R8_2PLANE_444_UNORM_EXT => todo!(),
                vk::Format::G10X6_B10X6R10X6_2PLANE_444_UNORM_3PACK16_EXT => todo!(),
                vk::Format::G12X4_B12X4R12X4_2PLANE_444_UNORM_3PACK16_EXT => todo!(),
                vk::Format::G16_B16R16_2PLANE_444_UNORM_EXT => todo!(),
                vk::Format::A4R4G4B4_UNORM_PACK16_EXT => todo!(),
                vk::Format::A4B4G4R4_UNORM_PACK16_EXT => todo!(),
                vk::Format::MAX_ENUM => todo!(),
            }
        } else {
            panic!()
        }
    }

    fn set_layout(&mut self, layout: ash::vk::ImageLayout) {}

    fn layout(&self) -> ash::vk::ImageLayout {
        todo!()
    }

    fn handle(&self) -> ash::vk::Image {
        todo!()
    }
}

pub struct SkiaGpuCanvas2D {
    context: RecordingContext,
    surfaces: Vec<Surface>,
    surface_images: Vec<Rc<RefCell<SkiaCanvasImage>>>,
    surface_image_views: Vec<ash::vk::ImageView>,
    current_image_index: usize,
}

impl SkiaGpuCanvas2D {
    pub fn new(
        device: &DeviceContext,
        queue: &CommandQueue,
        image_count: usize,
        width: u32,
        height: u32,
    ) -> Self {
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
                    device.handle().handle().as_raw() as _,
                    (
                        queue.handle().as_raw() as _,
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

        let surface_images: Vec<Rc<RefCell<SkiaCanvasImage>>> = surfaces
            .iter_mut()
            .map(|surface| {
                if let Some(t) =
                    surface.get_backend_texture(skia_safe::surface::BackendHandleAccess::FlushRead)
                {
                    SkiaCanvasImage::new(t, width, height);
                }

                panic!()
            })
            .collect();

        let surface_image_views: Vec<ash::vk::ImageView> = surface_images
            .iter()
            .map(|image| {
                let create_info = ash::vk::ImageViewCreateInfo::builder()
                    .image(image.borrow().handle())
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
                        .handle()
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

    fn flush(&mut self) -> (Rc<RefCell<SkiaCanvasImage>>, ash::vk::ImageView) {
        if let Some(direct) = self.context.as_direct_context().as_mut() {
            direct.flush_submit_and_sync_cpu();
        }

        let view = self.surface_image_views[self.current_image_index];
        let image = self.surface_images[self.current_image_index].clone();
        self.current_image_index += 1;
        self.current_image_index %= self.surface_images.len();

        (image, view)
    }
}
