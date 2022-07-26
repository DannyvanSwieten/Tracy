use crate::application::Application;
use crate::canvas_2d::Canvas2D;
use crate::image_renderer::ImageRenderer;
use crate::skia_vulkan_canvas::SkiaGpuCanvas2D;
use crate::window_delegate::WindowDelegate;

use ash::vk::QueueFlags;
use vk_utils::command_buffer::CommandBuffer;
use vk_utils::device_context::DeviceContext;
use vk_utils::queue::CommandQueue;
use vk_utils::renderpass::RenderPass;
use vk_utils::swapchain::Swapchain;
use vk_utils::wait_handle::WaitHandle;

use super::user_interface::{UIDelegate, UserInterface};
use super::window_event::MouseEvent;

use std::rc::Rc;

struct UI<AppState> {
    canvas: SkiaGpuCanvas2D,
    swapchain: Swapchain,
    user_interface: UserInterface<AppState>,
    image_renderer: ImageRenderer,
}

pub struct UIGpuDrawingWindowDelegate<AppState> {
    device: Rc<DeviceContext>,
    queue: Rc<CommandQueue>,
    renderpass: Option<RenderPass>,
    ui: Option<UI<AppState>>,
    ui_delegate: Box<dyn UIDelegate<AppState>>,
    fences: Vec<Vec<Option<WaitHandle>>>,
    sub_optimal_swapchain: bool,
}

impl<'a, AppState: 'static> UIGpuDrawingWindowDelegate<AppState> {
    pub fn new(device: Rc<DeviceContext>, ui_delegate: Box<dyn UIDelegate<AppState>>) -> Self {
        let queue = Rc::new(CommandQueue::new(device.clone(), QueueFlags::GRAPHICS));
        Self {
            device: device.clone(),
            queue,
            renderpass: None,
            ui: None,
            ui_delegate,
            fences: vec![Vec::new(), Vec::new(), Vec::new()],
            sub_optimal_swapchain: false,
        }
    }

    fn rebuild_swapchain(&mut self, _: &AppState) {
        self.device.wait();
        let new_swapchain = {
            if let Some(ui) = &self.ui {
                Swapchain::new(
                    self.device.clone(),
                    *ui.swapchain.surface(),
                    Some(&ui.swapchain),
                    self.queue.clone(),
                    ui.swapchain.logical_width(),
                    ui.swapchain.logical_height(),
                )
            } else {
                panic!()
            }
        };

        self.ui.as_mut().unwrap().swapchain = new_swapchain;
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

    fn mouse_dragged(&mut self, state: &mut AppState, x: f32, y: f32, dx: f32, dy: f32) {
        let p = skia_safe::Point::from((x, y));
        let d = skia_safe::Point::from((dx, dy));
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface
                .mouse_drag(state, &MouseEvent::new_with_delta(0, &p, &p, &d));
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
        self.device.wait();
        let (surface, old_swapchain) = if let Some(ui) = &self.ui {
            (*ui.swapchain.surface(), Some(&ui.swapchain))
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
            (surface, None)
        };

        let swapchain = Swapchain::new(
            self.device.clone(),
            surface,
            old_swapchain,
            self.queue.clone(),
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
            &self.queue,
            swapchain.image_count(),
            swapchain.physical_width(),
            swapchain.physical_height(),
        );

        user_interface.resize(
            state,
            swapchain.physical_width(),
            swapchain.physical_height(),
        );

        user_interface.resized(state);

        self.ui = Some(UI {
            canvas,
            swapchain,
            user_interface,
            image_renderer,
        });
    }

    fn update(&mut self, state: &mut AppState) {
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface.update(state)
        }
    }

    fn file_dropped(&mut self, state: &mut AppState, path: &std::path::PathBuf, x: f32, y: f32) {
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface
                .file_dropped(state, path, &skia_safe::Point::new(x, y))
        }
    }

    fn file_hovered(&mut self, state: &mut AppState, path: &std::path::PathBuf, x: f32, y: f32) {
        if let Some(ui) = self.ui.as_mut() {
            ui.user_interface
                .file_hovered(state, path, &skia_safe::Point::new(x, y))
        }
    }

    fn draw(&mut self, _: &Application<AppState>, state: &AppState) {
        // draw user interface

        if self.ui.is_none() {
            return;
        }

        if self.sub_optimal_swapchain {
            self.rebuild_swapchain(state)
        }

        let (mut image, view, (sub_optimal, index, framebuffer, semaphore)) = {
            if let Some(ui) = self.ui.as_mut() {
                ui.user_interface.paint(state, &mut ui.canvas);
                let (image, image_view) = ui.canvas.flush();
                (
                    image,
                    image_view,
                    ui.swapchain
                        .next_frame_buffer()
                        .expect("Acquire next image failed"),
                )
            } else {
                return;
            }
        };

        let mut command_buffer = CommandBuffer::new(self.device.clone(), self.queue.clone());
        {
            let image_ref = Rc::get_mut(&mut image).unwrap().get_mut();
            command_buffer.begin();

            self.sub_optimal_swapchain = sub_optimal;
            self.fences[index as usize].clear();
            if let Some(ui) = self.ui.as_mut() {
                command_buffer.image_resource_transition(
                    image_ref,
                    ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );

                if let Some(renderpass) = &self.renderpass {
                    command_buffer.begin_render_pass(
                        renderpass,
                        &framebuffer,
                        ui.swapchain.physical_width(),
                        ui.swapchain.physical_height(),
                    );

                    ui.image_renderer
                        .render(&command_buffer, &view, index as usize);
                }

                command_buffer.image_resource_transition(
                    image_ref,
                    ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                );

                command_buffer.end();
                self.fences[index as usize].push(Some(command_buffer.submit()));

                ui.swapchain.swap(&semaphore, index);
            }
        }
    }
}
