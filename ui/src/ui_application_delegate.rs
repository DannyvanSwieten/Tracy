use crate::application::{Application, ApplicationDelegate, WindowRegistry};
use crate::application_model::ApplicationModel;
use crate::ui_gpu_drawing_window_delegate::UIGpuDrawingWindowDelegate;
use crate::user_interface::UIBuilder;
use crate::window_delegate::WindowDelegate;
use std::rc::Rc;
use vk_utils::device_context::DeviceContext;
use winit::event_loop::EventLoopWindowTarget;

struct WindowRequest<Model> {
    title: String,
    width: u32,
    height: u32,
    ui_delegate: Box<dyn UIBuilder<Model>>,
}

pub struct UIApplicationDelegate<Model: ApplicationModel> {
    on_start: Option<Box<dyn FnMut(&Application<Model>, &mut Model)>>,
    on_update: Option<Box<dyn FnMut(&Application<Model>, &mut Model)>>,
    on_device_created: Option<Box<dyn FnMut(&vk_utils::gpu::Gpu, Rc<DeviceContext>, &mut Model)>>,
    device_builder:
        Option<Box<dyn FnMut(&vk_utils::gpu::Gpu, Vec<&'static std::ffi::CStr>) -> DeviceContext>>,
    window_requests: Vec<WindowRequest<Model>>,
    _state: std::marker::PhantomData<Model>,
}

impl<Model: ApplicationModel> UIApplicationDelegate<Model> {
    pub fn new() -> Self {
        Self {
            on_start: None,
            on_update: None,
            on_device_created: None,
            device_builder: None,
            window_requests: Vec::new(),
            _state: std::marker::PhantomData::default(),
        }
    }

    pub fn on_start<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Application<Model>, &mut Model) + 'static,
    {
        self.on_start = Some(Box::new(f));
        self
    }

    pub fn on_update<F>(mut self, f: F) -> Self
    where
        F: FnMut(&Application<Model>, &mut Model) + 'static,
    {
        self.on_update = Some(Box::new(f));
        self
    }

    pub fn on_device_created<F>(mut self, f: F) -> Self
    where
        F: FnMut(&vk_utils::gpu::Gpu, Rc<DeviceContext>, &mut Model) + 'static,
    {
        self.on_device_created = Some(Box::new(f));
        self
    }

    pub fn with_device_builder<F>(mut self, f: F) -> Self
    where
        F: FnMut(&vk_utils::gpu::Gpu, Vec<&'static std::ffi::CStr>) -> DeviceContext + 'static,
    {
        self.device_builder = Some(Box::new(f));
        self
    }

    pub fn with_window<D>(mut self, title: &str, width: u32, height: u32, ui_delegate: D) -> Self
    where
        D: UIBuilder<Model> + 'static,
    {
        self.window_requests.push(WindowRequest {
            title: title.to_string(),
            width,
            height,
            ui_delegate: Box::new(ui_delegate),
        });
        self
    }
}

impl<Model: ApplicationModel> ApplicationDelegate<Model> for UIApplicationDelegate<Model> {
    fn application_will_start(
        &mut self,
        app: &Application<Model>,
        state: &mut Model,
        window_registry: &mut WindowRegistry<Model>,
        target: &EventLoopWindowTarget<()>,
    ) {
        if let Some(cb) = self.on_start.as_mut() {
            cb(app, state)
        }

        while self.window_requests.len() != 0 {
            if let Some(request) = self.window_requests.pop() {
                let window = window_registry.create_window(
                    target,
                    &request.title,
                    request.width,
                    request.height,
                );

                let device_extensions = vec![ash::extensions::khr::Swapchain::name()];
                let gpu = &app
                    .vulkan()
                    .hardware_devices_with_queue_support(ash::vk::QueueFlags::GRAPHICS)[0];
                let device = {
                    if let Some(cb) = self.device_builder.as_mut() {
                        Rc::new(cb(&gpu, device_extensions))
                    } else {
                        Rc::new(gpu.device_context(&device_extensions, |builder| builder))
                    }
                };
                if let Some(cb) = self.on_device_created.as_mut() {
                    cb(&gpu, device.clone(), state)
                }
                let mut window_delegate =
                    UIGpuDrawingWindowDelegate::new(device, request.ui_delegate);
                window_delegate.resized(&window, app, state, request.width, request.height);
                window_registry.register_with_delegate(window, Box::new(window_delegate));
            }
        }
    }

    fn application_will_update(
        &mut self,
        app: &Application<Model>,
        state: &mut Model,
        _: &mut WindowRegistry<Model>,
        _: &EventLoopWindowTarget<()>,
    ) {
        if let Some(cb) = self.on_update.as_mut() {
            cb(app, state)
        }
    }
}
