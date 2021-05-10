use crate::application::Application;

pub trait WindowDelegate<AppState> {
    fn mouse_moved(&mut self, state: &mut AppState, event: &winit::dpi::PhysicalPosition<f64>) {}
    fn mouse_dragged(&mut self, state: &mut AppState, event: &winit::dpi::PhysicalPosition<f64>) {}
    fn mouse_down(&mut self, state: &mut AppState, event: &winit::dpi::PhysicalPosition<f64>) {}
    fn mouse_up(&mut self, state: &mut AppState, event: &winit::dpi::PhysicalPosition<f64>) {}
    fn resized(
        &mut self,
        window: &winit::window::Window,
        app: &Application<AppState>,
        state: &mut AppState,
        size: &winit::dpi::PhysicalSize<u32>,
    ) {
    }
    fn keyboard_event(&mut self, state: &mut AppState, event: &winit::event::KeyboardInput) {}
    fn draw(&mut self, app: &Application<AppState>, state: &AppState) {}

    fn update(&mut self, state: &mut AppState) {}
}