use std::ffi::CString;

use crate::node::*;
use crate::user_interface::*;
use skia_safe::canvas::Canvas;
use skia_safe::Point;

#[derive(PartialEq, Eq, Hash)]
pub enum MouseEventType {
    MouseDown,
    MouseUp,
    MouseMove,

    DoubleClick,
}

#[repr(C)]
pub struct MouseEvent {
    modifiers: u32,
    global_position: Point,
    local_position: Point,
}

impl MouseEvent {
    pub fn new(modifiers: u32, global_position: &Point, local_position: &Point) -> Self {
        Self {
            modifiers,
            global_position: *global_position,
            local_position: *local_position,
        }
    }

    pub fn is_control_down(&self) -> bool {
        (self.modifiers & 1) != 0
    }

    pub fn is_shift_down(&self) -> bool {
        (self.modifiers & 2) != 0
    }

    pub fn is_right_mouse(&self) -> bool {
        (self.modifiers & 4) != 0
    }

    pub fn global_position(&self) -> &Point {
        &self.global_position
    }

    pub fn local_position(&self) -> &Point {
        &self.local_position
    }
}

#[repr(C)]
pub struct Window<AppState> {
    uid: usize,
    name: String,
    width: u32,
    height: u32,
    pub ui: Option<UserInterface<AppState>>,
}

impl<AppState: 'static> Window<AppState> {
    pub fn new(name: &str) -> Window<AppState> {
        Window {
            uid: 0,
            name: name.to_string(),
            width: 0,
            height: 0,
            ui: None,
        }
    }

    pub fn update(&mut self, state: &mut AppState) {
        if let Some(ui) = self.ui.as_mut() {
            ui.update(state)
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> usize {
        return self.uid;
    }

    pub fn layout(&mut self, state: &mut AppState) {
        if let Some(ui) = self.ui.as_mut() {
            ui.resize(state, self.width, self.height)
        }
    }

    pub fn create_user_interface(&mut self, root: Node<AppState>) {
        self.ui = Some(UserInterface::new(root))
    }

    pub fn resized(&mut self, state: &mut AppState, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        if let Some(ui) = self.ui.as_mut() {
            ui.resize(state, width, height);
        }
    }

    pub fn mouse_down(&mut self, state: &mut AppState, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.mouse_down(state, event)
        }
    }

    pub fn mouse_up(&mut self, state: &mut AppState, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.mouse_up(state, event)
        }
    }

    pub fn double_click(&mut self, state: &mut AppState, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.double_click(state, event);
        }
    }

    pub fn mouse_drag(&mut self, state: &mut AppState, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.mouse_drag(state, event);
        }
    }

    pub fn mouse_enter(&mut self, state: &mut AppState, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.mouse_moved(state, event);
        }
    }

    pub fn mouse_leave(&mut self, state: &mut AppState, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.mouse_leave(state, event);
        }
    }

    pub fn render<'a>(&mut self, state: &mut AppState, canvas: &mut Canvas) {
        if let Some(ui) = self.ui.as_mut() {
            ui.paint(state, canvas);
        }
    }
}
