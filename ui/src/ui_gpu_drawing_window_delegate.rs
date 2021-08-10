use crate::application::Application;
use crate::canvas_2d::Canvas2D;
use crate::image_renderer::ImageRenderer;
use crate::skia_vulkan_canvas::SkiaGpuCanvas2D;
use crate::window_delegate::WindowDelegate;

use vk_utils::device_context::DeviceContext;
use vk_utils::swapchain::Swapchain;

use super::user_interface::{UIDelegate, UserInterface};
use super::window_event::MouseEvent;

struct UI<AppState> {
    canvas: SkiaGpuCanvas2D,
    swapchain: Swapchain,
    user_interface: UserInterface<AppState>,
    image_renderer: ImageRenderer,
}

pub struct UIGpuDrawingWindowDelegate<AppState> {
    device: DeviceContext,
    ui: Option<UI<AppState>>,
    ui_delegate: Box<dyn UIDelegate<AppState>>,
}

impl<'a, AppState: 'static> UIGpuDrawingWindowDelegate<AppState> {
    pub fn new(device: DeviceContext, ui_delegate: Box<dyn UIDelegate<AppState>>) -> Self {
        Self {
            device,
            ui: None,
            ui_delegate,
        }
    }
}

impl<'a, AppState: 'static> WindowDelegate<AppState> for UIGpuDrawingWindowDelegate<AppState> {
    fn mouse_moved(&mut self, state: &mut AppState, x: f32, y: f32) {
        let p = skia_safe::Point::from((x, y));
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface
                .mouse_moved(state, &MouseEvent::new(0, &p, &p));
        }
    }

    fn mouse_dragged(&mut self, state: &mut AppState, x: f32, y: f32) {
        let p = skia_safe::Point::from((x, y));
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface
                .mouse_drag(state, &MouseEvent::new(0, &p, &p));
        }
    }

    fn mouse_down(&mut self, state: &mut AppState, x: f32, y: f32) {
        let p = skia_safe::Point::from((x, y));
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface
                .mouse_down(state, &MouseEvent::new(0, &p, &p));
        }
    }

    fn mouse_up(&mut self, state: &mut AppState, x: f32, y: f32) {
        let p = skia_safe::Point::from((x, y));
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface
                .mouse_up(state, &MouseEvent::new(0, &p, &p));
        }
    }

    fn resized(
        &mut self,
        window: &winit::window::Window,
        _app: &Application<AppState>,
        state: &mut AppState,
        width: u32,
        height: u32,
    ) {
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface.resize(state, width, height);
        } else {
            let surface = unsafe {
                ash_window::create_surface(
                    self.device.gpu().vulkan().library(),
                    self.device.gpu().vulkan().vk_instance(),
                    window,
                    None,
                )
                .expect("Surface creation failed")
            };
            let swapchain = Swapchain::new(
                self.device.gpu().vulkan(),
                self.device.gpu(),
                &self.device,
                surface,
                None,
                0,
                width,
                height,
            );
            let mut user_interface = UserInterface::new(self.ui_delegate.build("root", state));
            let image_renderer = ImageRenderer::new(
                &self.device,
                swapchain.render_pass(),
                swapchain.image_count(),
                swapchain.physical_width(),
                swapchain.physical_height(),
            );
            let canvas = SkiaGpuCanvas2D::new(
                &self.device,
                swapchain.image_count(),
                swapchain.physical_width(),
                swapchain.physical_height(),
            );
            user_interface.resize(
                state,
                swapchain.physical_width(),
                swapchain.physical_height(),
            );

            self.ui = Some(UI {
                canvas,
                swapchain,
                user_interface,
                image_renderer,
            });
        }
    }

    fn update(&mut self, state: &mut AppState) {
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface.update(state)
        }
    }

    fn draw(&mut self, _: &Application<AppState>, state: &AppState) {
        // draw user interface
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface.paint(state, &mut ui.canvas);
            let (image, image_view) = ui.canvas.flush();

            let (sub_optimal, index, framebuffer, semaphore) = ui
                .swapchain
                .next_frame_buffer()
                .expect("Acquire next image failed");

            if let Some(queue) = self.device.graphics_queue() {
                queue.begin(|commands| {
                    commands.color_image_transition(
                        &image,
                        ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    );

                    commands
                });

                queue.begin_render_pass(
                    ui.swapchain.render_pass(),
                    &framebuffer,
                    ui.swapchain.physical_width(),
                    ui.swapchain.physical_height(),
                    |commands| {
                        ui.image_renderer
                            .render(&commands, &image_view, index as usize);
                        commands
                    },
                );

                queue.begin(|commands| {
                    commands.color_image_transition(
                        &image,
                        ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    );

                    commands
                });
            }

            ui.swapchain.swap(
                self.device.graphics_queue().as_ref().unwrap().vk_queue(),
                &semaphore,
                index,
            );
        }
    }
}
