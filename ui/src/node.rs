use skia_safe::{Point, Rect, Size};

use crate::canvas_2d::Canvas2D;
use crate::widget::*;
use crate::window_event::{MouseEvent, MouseEventType};
use std::collections::HashMap;

static mut NODE_ID: u32 = 0;

fn next_node_id() -> u32 {
    unsafe {
        NODE_ID = NODE_ID + 1;
        NODE_ID
    }
}

#[repr(C)]
pub struct Node<AppState> {
    uid: u32,
    material_tag: String,
    name: String,
    pub rect: Rect,
    padding: f32,
    spacing: f32,
    pub constraints: Constraints,
    widget: Box<dyn Widget<AppState>>,
    children: Vec<Node<AppState>>,
    style: Material,
    preferred_width: Option<f32>,
    preferred_height: Option<f32>,
    flex: Option<f32>,

    mouse_callbacks:
        HashMap<MouseEventType, Box<dyn FnMut(&MouseEvent, &mut AppState) -> Action<AppState>>>,
    build_callback: Option<Box<dyn FnMut(&AppState) -> Option<Vec<Node<AppState>>>>>,
    file_drop_handler:
        Option<Box<dyn FnMut(&mut AppState, &std::path::PathBuf) -> Action<AppState>>>,
}

impl<AppState> Node<AppState> {
    pub fn new(tag: &str) -> Self {
        Node {
            name: String::from(""),
            material_tag: tag.to_string(),
            rect: Rect::default(),
            padding: 0.,
            spacing: 0.,
            preferred_width: None,
            preferred_height: None,
            flex: None,
            constraints: Constraints::default(),
            widget: Box::new(Container::new()),
            children: Vec::new(),
            uid: next_node_id(),
            mouse_callbacks: HashMap::new(),
            build_callback: None,
            file_drop_handler: None,
            style: Material::new(),
        }
    }

    pub fn widget<T>(mut self, w: T) -> Self
    where
        T: Widget<AppState> + 'static,
    {
        self.widget = Box::new(w);
        self
    }

    pub fn with_constraints(mut self, constraints: Constraints) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn relative_size(mut self, width: f32, height: f32) -> Self {
        self.constraints.max_width = width;
        self.constraints.max_height = height;
        self
    }

    pub fn with_preferred_width(mut self, width: f32) -> Self {
        self.preferred_width = Some(width);
        self
    }

    pub fn with_preferred_height(mut self, height: f32) -> Self {
        self.preferred_height = Some(height);
        self
    }

    pub fn with_preferred_size(mut self, width: f32, height: f32) -> Self {
        self.preferred_width = Some(width);
        self.preferred_height = Some(height);
        self
    }

    pub fn with_flex(mut self, factor: f32) -> Self {
        self.flex = Some(factor);
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_mouse_event_callback<F>(mut self, event: MouseEventType, cb: F) -> Self
    where
        F: FnMut(&MouseEvent, &mut AppState) -> Action<AppState> + 'static,
    {
        self.mouse_callbacks.insert(event, Box::new(cb));
        self
    }

    pub fn flex(&self) -> Option<f32> {
        self.flex
    }

    pub fn preferred_width(&self) -> Option<f32> {
        self.preferred_width
    }

    pub fn on_rebuild<F>(mut self, cb: F) -> Self
    where
        F: FnMut(&AppState) -> Option<Vec<Node<AppState>>> + 'static,
    {
        self.build_callback = Some(Box::new(cb));
        self
    }

    pub fn on_file_drop<F>(mut self, handler: F) -> Self
    where
        F: FnMut(&mut AppState, &std::path::PathBuf) -> Action<AppState> + 'static,
    {
        self.file_drop_handler = Some(Box::new(handler));
        self
    }

    pub fn add_child(&mut self, child: Node<AppState>) -> &mut Self {
        self.children.push(child);
        return self;
    }

    pub fn child(mut self, child: Node<AppState>) -> Self {
        self.children.push(child);
        return self;
    }

    pub fn get_child_by_uid(&mut self, uid: u32) -> Option<&mut Self> {
        if self.uid == uid {
            return Some(self);
        } else {
            for child in self.children.iter_mut() {
                if let Some(c) = child.get_child_by_uid(uid) {
                    return Some(c);
                }
            }
        }

        None
    }

    pub fn resized(&mut self, state: &mut AppState) {
        for child in self.children.iter_mut() {
            child.resized(state);
        }
    }

    pub fn send_mouse_enter(&mut self, state: &mut AppState, uid: u32, event: &MouseEvent) {
        if self.uid == uid {
            self.widget.mouse_enter(state, &self.rect, event);
        } else {
            for child in self.children.iter_mut() {
                child.send_mouse_enter(state, uid, event);
            }
        }
    }

    pub fn send_mouse_leave(&mut self, state: &mut AppState, uid: u32, event: &MouseEvent) {
        if self.uid == uid {
            self.widget.mouse_leave(state, &self.rect, event);
        } else {
            for child in self.children.iter_mut() {
                child.send_mouse_leave(state, uid, event);
            }
        }
    }

    pub fn file_dropped(
        &mut self,
        state: &mut AppState,
        file: &std::path::PathBuf,
        position: &Point,
    ) -> Action<AppState> {
        let mut action = Action::None;

        if self.hit_test(position) {
            let mut consume = true;
            for child in self.children.iter_mut() {
                if child.hit_test(position) {
                    action = child.file_dropped(state, file, position);
                    consume = false;
                }
            }

            if consume {
                if let Some(cb) = &mut self.file_drop_handler {
                    action = cb(state, file);
                }
            }
        }

        action
    }

    pub fn prefrerred_width(&self) -> Option<f32> {
        self.preferred_width
    }

    pub fn preferred_height(&self) -> Option<f32> {
        self.preferred_height
    }

    pub fn calculate_size(&mut self, constraints: &Constraints) {
        let (size, children_constraints) = self.widget.calculate_size(
            self.preferred_width,
            self.preferred_height,
            constraints,
            &self.children,
        );

        assert_ne!(size.width, 0.0);

        self.set_size(&size);

        for child in 0..self.children.len() {
            self.children[child].calculate_size(&children_constraints[child])
        }
    }

    pub fn layout(&mut self, state: &AppState) {
        self.widget.layout(
            state,
            &self.rect,
            self.spacing,
            self.padding,
            &mut self.children,
        );
        for child in self.children.iter_mut() {
            child.layout(state);
        }
    }

    pub fn layout_child_with_name(&mut self, name: &str, state: &AppState) {
        if self.name == name {
            self.layout(state)
        } else {
            for child in self.children.iter_mut() {
                child.layout_child_with_name(name, state);
            }
        }
    }

    pub fn build(&mut self, state: &AppState) {
        if let Some(cb) = self.build_callback.as_mut() {
            if let Some(children) = cb(state) {
                self.children = children;
            } else {
                self.children.clear();
            }
        }

        for child in self.children.iter_mut() {
            child.build(state);
        }
    }

    pub fn set_size(&mut self, size: &Size) {
        self.rect.set_wh(size.width, size.height);
    }

    pub fn draw(&mut self, state: &AppState, canvas: &mut dyn Canvas2D, material: &Material) {
        self.widget.paint(
            state,
            &self.rect,
            canvas,
            material
                .get_child(&self.material_tag)
                .unwrap_or(&StyleSheet::default()),
        );

        for child in self.children.iter_mut() {
            child.draw(state, canvas, material);
        }
    }

    pub fn hit_test(&self, pos: &Point) -> bool {
        let bx = pos.x >= self.rect.left && pos.x < self.rect.right;
        let by = pos.y >= self.rect.top && pos.y < self.rect.bottom;
        bx && by
    }

    pub fn mouse_down(&mut self, state: &mut AppState, event: &MouseEvent) -> Action<AppState> {
        let mut action = Action::None;

        if self.hit_test(&event.global_position()) {
            let mut consume = true;
            for child in self.children.iter_mut() {
                if child.hit_test(&event.global_position()) {
                    action = child.mouse_down(state, event);
                    consume = false;
                }
            }

            if consume {
                self.widget.mouse_down(state, &self.rect, event);
                if let Some(cb) = self.mouse_callbacks.get_mut(&MouseEventType::MouseDown) {
                    action = cb(event, state);
                }
            }
        }

        action
    }

    pub fn mouse_up(&mut self, state: &mut AppState, event: &MouseEvent) -> Action<AppState> {
        let mut action = Action::None;

        if self.hit_test(&event.global_position()) {
            let mut consume = true;
            for child in self.children.iter_mut() {
                if child.hit_test(&event.global_position()) {
                    action = child.mouse_up(state, event);
                    consume = false;
                }
            }

            if consume {
                action = self.widget.mouse_up(state, &self.rect, event);
                if let Some(cb) = self.mouse_callbacks.get_mut(&MouseEventType::MouseUp) {
                    action = cb(event, state);
                }
            }
        }

        action
    }

    pub fn double_click(&mut self, state: &mut AppState, event: &MouseEvent) {
        if self.hit_test(&event.global_position()) {
            let mut consume = true;
            for child in self.children.iter_mut() {
                if child.hit_test(&event.global_position()) {
                    child.double_click(state, event);
                    consume = false;
                }
            }

            if consume {
                self.widget.double_click(state, &self.rect, event);
            }
        }
    }

    pub fn mouse_drag(&mut self, state: &mut AppState, event: &MouseEvent) -> Action<AppState> {
        if self.hit_test(&event.global_position()) {
            let mut consume = true;
            for child in self.children.iter_mut() {
                if child.hit_test(&event.global_position()) {
                    child.mouse_drag(state, event);
                    consume = false;
                }
            }

            if consume {
                self.widget.mouse_drag(state, &self.rect, event);
            }
        }

        Action::None
    }

    pub fn mouse_moved(&mut self, state: &mut AppState, event: &MouseEvent) -> Option<u32> {
        for child in self.children.iter_mut() {
            if child.hit_test(&event.global_position()) {
                return child.mouse_moved(state, event);
            }
        }

        return Some(self.uid);
    }

    pub fn mouse_leave(&mut self, state: &mut AppState, event: &MouseEvent) {
        self.widget.mouse_leave(state, &self.rect, event);
    }
}
