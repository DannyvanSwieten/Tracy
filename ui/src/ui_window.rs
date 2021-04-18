use super::application::*;
use super::node::Node;
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
    fn mouse_down(&mut self, event: &winit::dpi::PhysicalPosition<f64>) {}
    fn mouse_up(&mut self, event: &winit::dpi::PhysicalPosition<f64>) {}
    fn keyboard_event(&mut self, event: &winit::event::KeyboardInput) {}
}

pub struct UIWindow<AppState> {
    context: RecordingContext,
    surface: Surface,

    state: std::marker::PhantomData<AppState>,
    root: Option<Node<AppState>>,
}

impl<AppState: 'static> UIWindow<AppState> {
    pub fn new(app: &Application<AppState>, window: &winit::window::Window) -> Self {
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

        Self {
            context,
            surface,
            state: std::marker::PhantomData::<AppState>::default(),
            root: None,
        }
    }
}

impl<AppState: 'static> WindowDelegate<AppState> for UIWindow<AppState> {
    fn mouse_moved(&mut self, state: &mut AppState, event: &winit::dpi::PhysicalPosition<f64>) {
        if let Some(root) = &mut self.root {
            let p = skia_safe::Point::from((event.x as f32, event.y as f32));
            root.mouse_moved(state, &MouseEvent::new(0, &p, &p));
        }
    }
}
