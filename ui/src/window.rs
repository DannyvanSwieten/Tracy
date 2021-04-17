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
    pub modifiers: u32,
    pub global_position: Point,
    pub local_position: Point,
    pub delta_position: Point,
}

impl MouseEvent {
    pub fn is_control_down(&self) -> bool {
        (self.modifiers & 1) != 0
    }

    pub fn is_shift_down(&self) -> bool {
        (self.modifiers & 2) != 0
    }

    pub fn is_right_mouse(&self) -> bool {
        (self.modifiers & 4) != 0
    }
}

pub trait WindowDelegate<DataModel> {
    fn wants_user_interface(&self) -> bool;
    fn create_dom(&self, state: &DataModel) -> Node<DataModel>;
}

#[repr(C)]
pub struct Window<DataModel> {
    uid: usize,
    name: String,
    width: u32,
    height: u32,
    delegate: Option<Box<dyn WindowDelegate<DataModel>>>,
    pub ui: Option<UserInterface<DataModel>>,
}

impl<DataModel: 'static> Window<DataModel> {
    pub fn new(name: &str) -> Window<DataModel> {
        unsafe {
            let ptr = CString::new(name).expect("Failed");

            Window {
                uid: 0,
                name: name.to_string(),
                width: 0,
                height: 0,
                delegate: None,
                ui: None,
            }
        }
    }

    pub fn update(&mut self, state: &mut DataModel) {
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

    pub fn layout(&mut self, state: &mut DataModel) {
        if let Some(ui) = self.ui.as_mut() {
            ui.resize(state, self.width, self.height)
        }
    }

    pub fn create_user_interface(&mut self, root: Node<DataModel>) {
        self.ui = Some(UserInterface::new(root))
    }

    pub fn resized(&mut self, state: &mut DataModel, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        if let Some(ui) = self.ui.as_mut() {
            ui.resize(state, width, height);
        }
    }

    pub fn mouse_down(&mut self, state: &mut DataModel, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.mouse_down(state, event)
        }
    }

    pub fn mouse_up(&mut self, state: &mut DataModel, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.mouse_up(state, self.uid, event)
        }
    }

    pub fn double_click(&mut self, state: &mut DataModel, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.double_click(state, event);
        }
    }

    pub fn mouse_drag(&mut self, state: &mut DataModel, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.mouse_drag(state, event);
        }
    }

    pub fn mouse_enter(&mut self, state: &mut DataModel, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.mouse_moved(state, event);
        }
    }

    pub fn mouse_leave(&mut self, state: &mut DataModel, event: &MouseEvent) {
        if let Some(ui) = self.ui.as_mut() {
            ui.mouse_leave(state, event);
        }
    }

    pub fn render<'a>(&mut self, state: &mut DataModel, canvas: &mut Canvas) {
        if let Some(ui) = self.ui.as_mut() {
            ui.paint(state, canvas);
        }
    }
}
