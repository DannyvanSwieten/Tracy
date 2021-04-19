use super::application::*;
use super::node::Node;
use super::swapchain;
use super::window::MouseEvent;
use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk::Handle;
use skia_safe::gpu::*;
use skia_safe::{Budgeted, Canvas, ImageInfo, Surface};
use std::ffi::c_void;
use std::ptr;

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
    fn resized(&mut self, state: &mut AppState, size: &winit::dpi::PhysicalSize<f64>) {}
    fn keyboard_event(&mut self, state: &mut AppState, event: &winit::event::KeyboardInput) {}
}

pub struct UIWindow<'a, AppState> {
    context: RecordingContext,
    surface: Surface,
    vulkan_surface: ash::vk::SurfaceKHR,
    vulkan_surface_fn: ash::extensions::khr::Surface,

    state: std::marker::PhantomData<AppState>,
    root: Option<Node<AppState>>,
    device: &'a ash::Device,
    swapchain: swapchain::Swapchain<'a>,
    command_pool: ash::vk::CommandPool,
    command_buffers: Vec<ash::vk::CommandBuffer>,
    semaphores: Vec<ash::vk::Semaphore>,
}

impl<'a, AppState: 'static> UIWindow<'a, AppState> {
    pub fn new(
        app: &'a Application<AppState>,
        window: &winit::window::Window,
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
        let surface = Surface::new_render_target(
            &mut context,
            Budgeted::Yes,
            &image_info,
            None,
            SurfaceOrigin::TopLeft,
            None,
            false,
        )
        .unwrap();

        let vulkan_surface = unsafe { ash_window::create_surface(entry, instance, window, None) };
        match vulkan_surface {
            Ok(vs) => {
                let vulkan_surface_fn = ash::extensions::khr::Surface::new(entry, instance);
                let sc = swapchain::Swapchain::new(
                    instance,
                    app.primary_gpu(),
                    app.primary_device_context(),
                    app.present_queue_and_index().0,
                    &vulkan_surface_fn,
                    &vs,
                    app.present_queue_and_index().1 as u32,
                    window.inner_size().width,
                    window.inner_size().height,
                );

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

                Ok(Self {
                    context,
                    surface,
                    vulkan_surface_fn,
                    vulkan_surface: vs,
                    state: std::marker::PhantomData::<AppState>::default(),
                    root: None,
                    device: app.primary_device_context(),
                    swapchain: sc,
                    command_pool,
                    command_buffers,
                    semaphores,
                })
            }
            Err(_result) => Err("Swapchain creation failed"),
        }
    }

    pub fn draw(&self) {
        let clear_values = [ash::vk::ClearValue {
            color: ash::vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            },
        }];

        let (success, image_index, framebuffer) = self.swapchain.next_frame_buffer();

        if success {
            let render_pass_begin_info = ash::vk::RenderPassBeginInfo::builder()
                .render_pass(*self.swapchain.render_pass())
                .framebuffer(*framebuffer)
                .clear_values(&clear_values);
            let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::builder().build();

            let commands = &self.command_buffers[image_index as usize];
            unsafe {
                self.device
                    .begin_command_buffer(*commands, &command_buffer_begin_info)
                    .expect("Unable to start recording command buffer");

                self.device.cmd_begin_render_pass(
                    *commands,
                    &render_pass_begin_info,
                    ash::vk::SubpassContents::INLINE,
                );

                self.device.cmd_end_render_pass(*commands);
                self.device
                    .end_command_buffer(*commands)
                    .expect("End recording command buffer failed");

                self.swapchain
                    .swap(&self.semaphores[image_index as usize], image_index)
            }
        }
    }
}

impl<'a, AppState: 'static> WindowDelegate<AppState> for UIWindow<'a, AppState> {
    fn mouse_moved(&mut self, state: &mut AppState, event: &winit::dpi::PhysicalPosition<f64>) {
        if let Some(root) = &mut self.root {
            let p = skia_safe::Point::from((event.x as f32, event.y as f32));
            root.mouse_moved(state, &MouseEvent::new(0, &p, &p));
        }
    }

    fn resized(&mut self, _: &mut AppState, event: &winit::dpi::PhysicalSize<f64>) {
        let image_info = ImageInfo::new_n32_premul((event.width as i32, event.height as i32), None);

        self.surface = Surface::new_render_target(
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
