extern crate ash;
extern crate ash_window;
extern crate winit;

use crate::widget::Widget;
use crate::window_delegate::WindowDelegate;
use std::collections::{HashMap, VecDeque};
use vk_utils::vulkan::Vulkan;

use super::application_model::ApplicationModel;
use std::ffi::{CStr, CString};
use std::path::Path;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder, WindowId},
};

use ash::extensions::{ext::DebugUtils, khr::Surface};

#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;

#[cfg(target_os = "macos")]
use ash::vk::ExtMetalSurfaceFn;

#[cfg(target_os = "macos")]
pub fn surface_extension_name() -> &'static CStr {
    ExtMetalSurfaceFn::name()
}

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

#[cfg(target_os = "windows")]
pub fn surface_extension_name() -> &'static CStr {
    Win32Surface::name()
}

pub trait ApplicationDelegate<Model: ApplicationModel> {
    fn application_will_start(
        &mut self,
        _: &mut Application<Model>,
        _: &mut Model,
        _: &mut WindowRegistry<Model>,
        _: &EventLoopWindowTarget<()>,
    ) {
    }
    fn application_will_quit(&mut self, _: &mut Application<Model>, _: &EventLoopWindowTarget<()>) {
    }

    fn application_will_update(
        &mut self,
        _: &Application<Model>,
        _: &mut Model,
        _: &mut WindowRegistry<Model>,
        _: &EventLoopWindowTarget<()>,
    ) {
    }

    fn window_requested(
        &mut self,
        _: &Application<Model>,
        _: &mut Model,
        _: &EventLoopWindowTarget<()>,
        _: &mut WindowRegistry<Model>,
        _: WindowRequest<Model>,
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
        _: &Application<Model>,
        _: &Model,
        _: &winit::window::WindowId,
    ) -> ControlFlow {
        ControlFlow::Wait
    }

    fn file_dropped(&mut self, _: &winit::window::WindowId, _: &Path) -> ControlFlow {
        ControlFlow::Wait
    }
}

pub struct WindowRegistry<Model: 'static> {
    windows: HashMap<WindowId, Window>,
    window_delegates: HashMap<WindowId, Box<dyn WindowDelegate<Model>>>,
}

impl<Model: ApplicationModel> WindowRegistry<Model> {
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

    pub fn register_with_delegate(
        &mut self,
        window: Window,
        delegate: Box<dyn WindowDelegate<Model>>,
    ) {
        self.window_delegates.insert(window.id(), delegate);
        self.windows.insert(window.id(), window);
    }

    pub fn register(&mut self, window: Window) {
        self.windows.insert(window.id(), window);
    }

    pub fn active_window_count(&self) -> usize {
        self.windows.len()
    }

    fn update(&mut self, state: &mut Model) {
        for (_, delegate) in self.window_delegates.iter_mut() {
            delegate.update(state)
        }
    }

    fn window_resized(
        &mut self,
        app: &Application<Model>,
        state: &mut Model,
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

    fn character_received(&mut self, id: &WindowId, character: char, state: &mut Model) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.character_received(state, character)
        }
    }

    fn keyboard_event(
        &mut self,
        id: &WindowId,
        event: &winit::event::KeyboardInput,
        state: &mut Model,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.keyboard_event(state, event)
        }
    }

    fn close_button_pressed(&mut self, id: &WindowId, state: &mut Model) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            if delegate.close_button_pressed(state) {
                self.windows.remove(id);
            }
        }
    }

    fn mouse_moved(
        &mut self,
        state: &mut Model,
        id: &WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.mouse_moved(state, position.x as f32, position.y as f32);
        }
    }

    fn mouse_dragged(
        &mut self,
        state: &mut Model,
        id: &winit::window::WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
        delta: &winit::dpi::PhysicalPosition<f64>,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.mouse_dragged(
                state,
                position.x as f32,
                position.y as f32,
                delta.x as f32,
                delta.y as f32,
            );
        }
    }

    fn mouse_down(
        &mut self,
        app: &mut Application<Model>,
        state: &mut Model,
        id: &winit::window::WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.mouse_down(app, state, position.x as f32, position.y as f32);
        }
    }

    fn mouse_up(
        &mut self,
        app: &mut Application<Model>,
        state: &mut Model,
        id: &winit::window::WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.mouse_up(app, state, position.x as f32, position.y as f32);
        }
    }

    fn window_moved(&mut self, _: &winit::window::WindowId, _: &winit::dpi::PhysicalPosition<i32>) {
    }

    fn draw(&mut self, app: &Application<Model>, state: &mut Model) {
        for (_, delegate) in self.window_delegates.iter_mut() {
            delegate.draw(app, state)
        }
    }

    fn window_destroyed(&mut self, id: &WindowId) {
        self.window_delegates.remove(id);
    }

    fn file_dropped(
        &mut self,
        id: &WindowId,
        state: &mut Model,
        file: &Path,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.file_dropped(state, file, position.x as f32, position.y as f32)
        }
    }

    fn file_hovered(
        &mut self,
        id: &WindowId,
        state: &mut Model,
        file: &Path,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) {
        if let Some(delegate) = self.window_delegates.get_mut(id) {
            delegate.file_hovered(state, file, position.x as f32, position.y as f32)
        }
    }
}

pub struct WindowRequest<Model: ApplicationModel> {
    pub builder: Box<dyn Fn(&Model) -> Box<dyn Widget<Model>>>,
    pub title: Option<String>,
    pub width: u32,
    pub height: u32,
}

pub struct Application<Model: ApplicationModel> {
    vulkan: Vulkan,
    pending_messages: VecDeque<Model::MessageType>,
    pending_window_requests: VecDeque<WindowRequest<Model>>,
    _state: std::marker::PhantomData<Model>,
}

impl<Model: ApplicationModel + 'static> Application<Model> {
    pub fn new(name: &str) -> Self {
        let layers = [CString::new("VK_LAYER_KHRONOS_validation").expect("String Creation Failed")];
        let instance_extensions = [
            Surface::name(),
            surface_extension_name(),
            DebugUtils::name(),
        ];
        let vulkan = Vulkan::new(name, &layers, &instance_extensions);

        Self {
            vulkan,
            pending_messages: VecDeque::new(),
            pending_window_requests: VecDeque::new(),
            _state: std::marker::PhantomData::<Model>::default(),
        }
    }

    pub fn vulkan(&self) -> &Vulkan {
        &self.vulkan
    }

    pub fn send_message(&mut self, msg: Model::MessageType) {
        self.pending_messages.push_back(msg)
    }

    fn pop_message(&mut self) -> Option<Model::MessageType> {
        self.pending_messages.pop_front()
    }

    pub fn ui_window_request(&mut self, request: WindowRequest<Model>) {
        self.pending_window_requests.push_back(request)
    }

    pub fn run<Delegate>(mut self, delegate: Delegate, state: Model)
    where
        Delegate: ApplicationDelegate<Model> + 'static,
    {
        let mut s = state;
        let event_loop = EventLoop::new();
        let mut d = delegate;

        let mut window_registry = WindowRegistry {
            windows: HashMap::new(),
            window_delegates: HashMap::new(),
        };

        d.application_will_start(&mut self, &mut s, &mut window_registry, &event_loop);
        let mut last_mouse_position = winit::dpi::PhysicalPosition::<f64>::new(0., 0.);
        let mut last_file_drop: Vec<std::path::PathBuf> = Vec::new();
        let mut mouse_is_down = false;
        event_loop.run(move |e, event_loop, control_flow| {
            while let Some(msg) = self.pop_message() {
                s.handle_message(msg)
            }

            while let Some(request) = self.pending_window_requests.pop_front() {
                d.window_requested(&self, &mut s, event_loop, &mut window_registry, request)
            }
            *control_flow = winit::event_loop::ControlFlow::Poll;
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
                } => last_file_drop.push(path_buffer),
                Event::WindowEvent {
                    event: WindowEvent::HoveredFile(path_buffer),
                    window_id,
                } => window_registry.file_hovered(
                    &window_id,
                    &mut s,
                    &path_buffer,
                    &last_mouse_position,
                ),
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
                            ..
                        },
                    window_id,
                } => {
                    if mouse_is_down {
                        let delta = winit::dpi::PhysicalPosition::<f64>::new(
                            position.x - last_mouse_position.x,
                            position.y - last_mouse_position.y,
                        );
                        window_registry.mouse_dragged(&mut s, &window_id, &position, &delta)
                    } else {
                        window_registry.mouse_moved(&mut s, &window_id, &position)
                    }

                    if !last_file_drop.is_empty() {
                        window_registry.file_dropped(
                            &window_id,
                            &mut s,
                            &last_file_drop[0],
                            &position,
                        );

                        last_file_drop.clear();
                    }

                    last_mouse_position = position;
                }

                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::ReceivedCharacter(character),
                } => window_registry.character_received(&window_id, character, &mut s),

                Event::WindowEvent {
                    window_id,
                    event:
                        WindowEvent::KeyboardInput {
                            device_id,
                            input,
                            is_synthetic,
                        },
                } => window_registry.keyboard_event(&window_id, &input, &mut s),

                Event::WindowEvent {
                    event:
                        WindowEvent::MouseInput {
                            device_id,
                            state,
                            button,
                            ..
                        },
                    window_id,
                } => match state {
                    winit::event::ElementState::Pressed => {
                        mouse_is_down = true;
                        window_registry.mouse_down(
                            &mut self,
                            &mut s,
                            &window_id,
                            &last_mouse_position,
                        )
                    }
                    winit::event::ElementState::Released => {
                        mouse_is_down = false;
                        window_registry.mouse_up(
                            &mut self,
                            &mut s,
                            &window_id,
                            &last_mouse_position,
                        )
                    }
                },
                Event::MainEventsCleared => {
                    d.application_will_update(&self, &mut s, &mut window_registry, event_loop);
                    window_registry.update(&mut s);
                    window_registry.draw(&self, &mut s);
                }
                _ => (),
            }

            if let ControlFlow::Exit = *control_flow {
                d.application_will_quit(&mut self, event_loop)
            }
        });
    }
}
