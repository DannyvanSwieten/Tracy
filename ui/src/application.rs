extern crate ash;
extern crate ash_window;
extern crate winit;

use crate::window_delegate::WindowDelegate;
use std::collections::HashMap;
use vk_utils::{device_context::DeviceContext, gpu::Gpu, vk_instance::Vulkan};

use ash::vk::QueueFlags;

use std::ffi::{CStr, CString};
use std::path::PathBuf;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder, WindowId},
};

use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain},
};

#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;

#[cfg(target_os = "macos")]
use ash::vk::ExtMetalSurfaceFn;

#[cfg(target_os = "macos")]
fn surface_extension_name() -> &'static CStr {
    ExtMetalSurfaceFn::name()
}

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

#[cfg(target_os = "windows")]
fn surface_extension_name() -> &'static CStr {
    Win32Surface::name()
}

pub trait ApplicationDelegate<AppState> {
    fn application_will_start(
        &mut self,
        _: &Application<AppState>,
        _: &mut AppState,
        _: &mut WindowRegistry<AppState>,
        _: &EventLoopWindowTarget<()>,
    ) {
    }
    fn application_will_quit(
        &mut self,
        _: &mut Application<AppState>,
        _: &EventLoopWindowTarget<()>,
    ) {
    }

    fn application_will_update(
        &mut self,
        _: &Application<AppState>,
        _: &mut AppState,
        _: &mut WindowRegistry<AppState>,
        _: &EventLoopWindowTarget<()>,
    ) {
    }

    fn window_moved(
        &mut self,
        _: &winit::window::WindowId,
        _: &winit::dpi::PhysicalPosition<i32>,
    ) -> ControlFlow {
        ControlFlow::Wait
    }

    fn window_got_focus(&mut self, _: &winit::window::WindowId) -> ControlFlow {
        ControlFlow::Wait
    }
    fn window_lost_focus(&mut self, _: &winit::window::WindowId) -> ControlFlow {
        ControlFlow::Wait
    }

    fn window_requested_redraw(
        &mut self,
        _: &Application<AppState>,
        _: &AppState,
        _: &winit::window::WindowId,
    ) -> ControlFlow {
        ControlFlow::Wait
    }

    fn file_dropped(&mut self, _: &winit::window::WindowId, _: &PathBuf) -> ControlFlow {
        ControlFlow::Wait
    }
}

pub struct WindowRegistry<AppState> {
    windows: HashMap<WindowId, Window>,
    window_delegates: HashMap<WindowId, Box<dyn WindowDelegate<AppState>>>,
}

impl<AppState> WindowRegistry<AppState> {
    pub fn create_window(
        &self,
        target: &EventLoopWindowTarget<()>,
        title: &str,
        width: u32,
        height: u32,
    ) -> Window {
        WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize { width, height })
            .build(target)
            .unwrap()
    }

    pub fn register(&mut self, window: Window, delegate: Box<dyn WindowDelegate<AppState>>) {
        self.window_delegates.insert(window.id(), delegate);
        self.windows.insert(window.id(), window);
    }

    pub fn active_window_count(&self) -> usize {
        self.windows.len()
    }

    fn update(&mut self, state: &mut AppState) {
        for (_, delegate) in self.window_delegates.iter_mut() {
            delegate.update(state)
        }
    }

    fn window_resized(
        &mut self,
        app: &Application<AppState>,
        state: &mut AppState,
        id: &winit::window::WindowId,
        size: &winit::dpi::PhysicalSize<u32>,
    ) {
        if let Some(window) = self.window_delegates.get_mut(id) {
            window.resized(
                self.windows.get(id).unwrap(),
                app,
                state,
                size.width,
                size.height,
            )
        }
    }

    fn close_button_pressed(&mut self, id: &WindowId, state: &mut AppState) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            if delegate.close_button_pressed(state) {
                self.windows.remove(id);
            }
        }
    }

    fn mouse_moved(
        &mut self,
        state: &mut AppState,
        id: &WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.mouse_moved(state, position.x as f32, position.y as f32);
        }
    }

    fn mouse_dragged(
        &mut self,
        state: &mut AppState,
        id: &winit::window::WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.mouse_dragged(state, position.x as f32, position.y as f32);
        }
    }

    fn mouse_down(
        &mut self,
        state: &mut AppState,
        id: &winit::window::WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.mouse_down(state, position.x as f32, position.y as f32);
        }
    }

    fn mouse_up(
        &mut self,
        state: &mut AppState,
        id: &winit::window::WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.mouse_up(state, position.x as f32, position.y as f32);
        }
    }

    fn window_moved(&mut self, _: &winit::window::WindowId, _: &winit::dpi::PhysicalPosition<i32>) {
    }

    fn draw(&mut self, app: &Application<AppState>, state: &mut AppState) {
        for (_, delegate) in self.window_delegates.iter_mut() {
            delegate.draw(app, state)
        }
    }

    fn window_destroyed(&mut self, id: &WindowId) {
        self.window_delegates.remove(id);
    }

    fn file_dropped(&mut self, _: &WindowId, _: &PathBuf) {}
}

pub struct Application<AppState> {
    vulkan: Vulkan,
    primary_gpu: Gpu,
    primary_device_context: DeviceContext,
    vulkan_surface_ext: Surface,
    vulkan_swapchain_ext: Swapchain,
    _state: std::marker::PhantomData<AppState>,
}

impl<AppState: 'static> Application<AppState> {
    pub fn new(name: &str) -> Self {
        let layers = [CString::new("VK_LAYER_KHRONOS_validation").expect("String Creation Failed")];
        let instance_extensions = [
            Surface::name(),
            surface_extension_name(),
            DebugUtils::name(),
        ];
        let vulkan = Vulkan::new(name, &layers, &instance_extensions);

        let mut gpus = vulkan.hardware_devices_with_queue_support(QueueFlags::GRAPHICS);
        let primary_gpu = gpus.remove(0);
        let primary_device_context =
            primary_gpu.device_context(&[Swapchain::name()], |_builder| _builder);

        let vulkan_surface_ext = Surface::new(vulkan.library(), vulkan.vk_instance());
        let vulkan_swapchain_ext =
            Swapchain::new(vulkan.vk_instance(), primary_device_context.vk_device());
        Self {
            vulkan,
            primary_gpu,
            primary_device_context,
            vulkan_surface_ext,
            vulkan_swapchain_ext,
            _state: std::marker::PhantomData::<AppState>::default(),
        }
    }

    pub fn vulkan(&self) -> &Vulkan {
        &self.vulkan
    }

    pub fn primary_gpu(&self) -> &Gpu {
        &self.primary_gpu
    }

    pub fn primary_device_context(&self) -> &DeviceContext {
        &self.primary_device_context
    }

    pub fn surface_extension(&self) -> &ash::extensions::khr::Surface {
        &self.vulkan_surface_ext
    }

    pub fn swapchain_extension(&self) -> &ash::extensions::khr::Swapchain {
        &self.vulkan_swapchain_ext
    }

    pub fn run(mut self, delegate: Box<dyn ApplicationDelegate<AppState>>, state: AppState) {
        let mut s = state;
        let event_loop = EventLoop::new();
        let mut d = delegate;

        let mut window_registry = WindowRegistry {
            windows: HashMap::new(),
            window_delegates: HashMap::new(),
        };

        d.application_will_start(&self, &mut s, &mut window_registry, &event_loop);
        let mut last_mouse_position = winit::dpi::PhysicalPosition::<f64>::new(0., 0.);
        let mut mouse_is_down = false;
        event_loop.run(move |e, event_loop, control_flow| {
            *control_flow = ControlFlow::Poll;
            d.application_will_update(&self, &mut s, &mut window_registry, &event_loop);
            match e {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } => {
                    window_registry.close_button_pressed(&window_id, &mut s);
                    if window_registry.active_window_count() == 0 {
                        *control_flow = ControlFlow::Exit;
                    }
                }

                Event::WindowEvent {
                    event: WindowEvent::Destroyed,
                    window_id,
                } => window_registry.window_destroyed(&window_id),

                Event::WindowEvent {
                    event: WindowEvent::Moved(physical_position),
                    window_id,
                } => window_registry.window_moved(&window_id, &physical_position),

                Event::WindowEvent {
                    event: WindowEvent::Resized(physical_size),
                    window_id,
                } => window_registry.window_resized(&self, &mut s, &window_id, &physical_size),

                Event::WindowEvent {
                    event: WindowEvent::DroppedFile(path_buffer),
                    window_id,
                } => window_registry.file_dropped(&window_id, &path_buffer),

                Event::WindowEvent {
                    event: WindowEvent::Focused(f),
                    window_id,
                } => {
                    *control_flow = if f {
                        d.window_got_focus(&window_id)
                    } else {
                        d.window_lost_focus(&window_id)
                    }
                }

                Event::RedrawRequested(_window_id) => {
                    //window_registry.window_requested_redraw(&self, &s, &window_id)
                }

                Event::WindowEvent {
                    event:
                        WindowEvent::CursorMoved {
                            device_id,
                            position,
                            modifiers,
                        },
                    window_id,
                } => {
                    last_mouse_position = position;
                    if mouse_is_down {
                        window_registry.mouse_dragged(&mut s, &window_id, &position)
                    } else {
                        window_registry.mouse_moved(&mut s, &window_id, &position)
                    }
                }

                Event::WindowEvent {
                    event:
                        WindowEvent::MouseInput {
                            device_id,
                            state,
                            button,
                            modifiers,
                        },
                    window_id,
                } => match state {
                    winit::event::ElementState::Pressed => {
                        mouse_is_down = true;
                        window_registry.mouse_down(&mut s, &window_id, &last_mouse_position)
                    }
                    winit::event::ElementState::Released => {
                        mouse_is_down = false;
                        window_registry.mouse_up(&mut s, &window_id, &last_mouse_position)
                    }
                },

                _ => (),
            }

            match control_flow {
                ControlFlow::Exit => d.application_will_quit(&mut self, &event_loop),
                _ => {
                    window_registry.update(&mut s);
                    window_registry.draw(&mut self, &mut s);
                }
            }
        });
    }
}
