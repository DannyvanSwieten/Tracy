use crate::application::{Application, ApplicationDelegate, WindowRegistry};
use crate::ui_gpu_drawing_window_delegate::UIGpuDrawingWindowDelegate;
use crate::user_interface::UIDelegate;
use crate::window_delegate::WindowDelegate;
use winit::event_loop::EventLoopWindowTarget;

struct WindowRequest<AppState> {
    title: String,
    width: u32,
    height: u32,
    ui_delegate: Box<dyn UIDelegate<AppState>>,
}

pub struct UIApplicationDelegate<AppState> {
    on_start: Option<Box<dyn FnMut(&mut AppState)>>,
    window_requests: Vec<WindowRequest<AppState>>,
    _state: std::marker::PhantomData<AppState>,
}

impl<AppState> UIApplicationDelegate<AppState> {
    pub fn new() -> Self {
        Self {
            on_start: None,
            window_requests: Vec::new(),
            _state: std::marker::PhantomData::default(),
        }
    }

    pub fn on_start<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut AppState) + 'static,
    {
        self.on_start = Some(Box::new(f));
        self
    }

    pub fn with_window(
        mut self,
        title: &str,
        width: u32,
        height: u32,
        ui_delegate: Box<dyn UIDelegate<AppState>>,
    ) -> Self {
        self.window_requests.push(WindowRequest {
            title: title.to_string(),
            width,
            height,
            ui_delegate,
        });
        self
    }
}

impl<AppState> ApplicationDelegate<AppState> for UIApplicationDelegate<AppState> {
    fn application_will_start(
        &mut self,
        app: &Application<AppState>,
        state: &mut AppState,
        window_registry: &mut WindowRegistry<AppState>,
        target: &EventLoopWindowTarget<()>,
    ) {
        if let Some(cb) = self.on_start.as_mut() {
            cb(state)
        }

        while self.window_requests.len() != 0 {
            if let Some(request) = self.window_requests.pop() {
                let window = window_registry.create_window(
                    target,
                    &request.title,
                    request.width,
                    request.height,
                );

                let extensions = vec![ash::extensions::khr::Swapchain::name()];
                let gpu = &app
                    .vulkan()
                    .hardware_devices_with_queue_support(ash::vk::QueueFlags::GRAPHICS)[0];
                let device = gpu.device_context(&extensions, |builder| builder);
                let mut window_delegate =
                    UIGpuDrawingWindowDelegate::new(device, request.ui_delegate);
                window_delegate.resized(&window, app, state, request.width, request.height);
                window_registry.register_with_delegate(window, Box::new(window_delegate));
            }
        }
    }
}
