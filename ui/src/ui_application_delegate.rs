use crate::application::{Application, ApplicationDelegate, WindowRegistry, WindowRequest};
use crate::application_model::ApplicationModel;
use crate::ui_gpu_drawing_window_delegate::UIGpuDrawingWindowDelegate;
use crate::window_delegate::WindowDelegate;
use std::rc::Rc;
use vk_utils::device_context::DeviceContext;
use winit::event_loop::EventLoopWindowTarget;

pub struct UIApplicationDelegate<Model: ApplicationModel> {
    on_start: Option<Box<dyn FnMut(&mut Application<Model>, &mut Model)>>,
    on_update: Option<Box<dyn FnMut(&Application<Model>, &mut Model)>>,
    on_device_created: Option<Box<dyn FnMut(&vk_utils::gpu::Gpu, Rc<DeviceContext>, &mut Model)>>,
    device_builder:
        Option<Box<dyn FnMut(&vk_utils::gpu::Gpu, Vec<&'static std::ffi::CStr>) -> DeviceContext>>,
    _state: std::marker::PhantomData<Model>,
    device: Option<Rc<DeviceContext>>,
}

impl<Model: ApplicationModel> UIApplicationDelegate<Model> {
    pub fn new() -> Self {
        Self {
            on_start: None,
            on_update: None,
            on_device_created: None,
            device_builder: None,
            _state: std::marker::PhantomData::default(),
            device: None,
        }
    }

    pub fn on_start<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut Application<Model>, &mut Model) + 'static,
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
}

impl<Model: ApplicationModel> ApplicationDelegate<Model> for UIApplicationDelegate<Model> {
    fn application_will_start(
        &mut self,
        app: &mut Application<Model>,
        state: &mut Model,
        _: &mut WindowRegistry<Model>,
        _: &EventLoopWindowTarget<()>,
    ) {
        let gpu = &app
            .vulkan()
            .hardware_devices_with_queue_support(ash::vk::QueueFlags::GRAPHICS)[0];
        let device_extensions = vec![ash::extensions::khr::Swapchain::name()];
        let device = {
            if let Some(cb) = self.device_builder.as_mut() {
                Rc::new(cb(gpu, device_extensions))
            } else {
                Rc::new(gpu.device_context(&device_extensions, |builder| builder))
            }
        };

        self.device = Some(device);
        if let Some(cb) = self.on_start.as_mut() {
            cb(app, state)
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

    fn window_requested(
        &mut self,
        app: &Application<Model>,
        state: &mut Model,
        target: &EventLoopWindowTarget<()>,
        window_registry: &mut WindowRegistry<Model>,
        request: WindowRequest<Model>,
    ) {
        let window = window_registry.create_window(
            target,
            &request.title.unwrap_or_else(|| "Untitled".to_string()),
            request.width,
            request.height,
        );

        let mut window_delegate =
            UIGpuDrawingWindowDelegate::new(self.device.as_ref().unwrap().clone(), request.builder);
        window_delegate.resized(
            &window,
            app,
            state,
            window.inner_size().width,
            window.inner_size().height,
        );
        window_registry.register_with_delegate(window, Box::new(window_delegate));
    }
}
