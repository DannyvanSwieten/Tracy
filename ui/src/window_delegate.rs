use crate::application::Application;

pub trait WindowDelegate<AppState> {
    fn close_button_pressed(&mut self, state: &mut AppState) -> bool {
        true
    }
    fn mouse_moved(&mut self, state: &mut AppState, x: f32, y: f32) {}
    fn mouse_dragged(&mut self, state: &mut AppState, x: f32, y: f32) {}
    fn mouse_down(&mut self, state: &mut AppState, x: f32, y: f32) {}
    fn mouse_up(&mut self, state: &mut AppState, x: f32, y: f32) {}
    fn resized(
        &mut self,
        window: &winit::window::Window,
        app: &Application<AppState>,
        state: &mut AppState,
        width: u32,
        height: u32,
    ) {
    }
    fn keyboard_event(&mut self, state: &mut AppState, event: &winit::event::KeyboardInput) {}
    fn draw(&mut self, app: &Application<AppState>, state: &AppState) {}

    fn update(&mut self, state: &mut AppState) {}
}
