use crate::application::Application;
use crate::application_model::ApplicationModel;
use crate::canvas_2d::Canvas2D;
use crate::constraints::BoxConstraints;
use crate::style::StyleContext;
use crate::widget::*;
use crate::window_event::MouseEvent;
use skia_safe::{Point, Size};
use std::path::Path;
use winit::{event::KeyboardInput, window::WindowId};

pub struct WindowContext {
    origin: Point,
    size: Size,
    id: WindowId,
}

pub struct DragContext<Model> {
    dragged_widgets: Vec<Box<dyn Widget<Model>>>,
}

pub struct UserInterface<Model: ApplicationModel> {
    pub root: ChildSlot<Model>,
    pub style_ctx: StyleContext,
    actions: Vec<Action<Model>>,
    theme: String,
}

impl<Model: ApplicationModel + 'static> UserInterface<Model> {
    pub fn new(root: Box<dyn Widget<Model>>, theme: &str) -> Self {
        UserInterface {
            root: ChildSlot::new_with_box(root),
            style_ctx: StyleContext::new(),
            actions: Vec::new(),
            theme: theme.to_string(),
        }
    }

    pub fn file_dropped(&mut self, state: &mut Model, path: &Path, position: &Point) {
        // self.actions
        //     .push(self.root.file_dropped(state, path, position))
    }

    pub fn file_hovered(&mut self, state: &mut Model, path: &Path, position: &Point) {}

    fn build_popup(&mut self, request: PopupRequest<Model>, position: &Point) {}

    pub fn resize(&mut self, state: &Model, width: u32, height: u32) {
        let constraints = BoxConstraints::new().with_tight_constraints(width as f32, height as f32);
        self.layout(&constraints, state);
    }

    pub fn resized(&mut self, state: &mut Model) {}

    pub fn mouse_down(
        &mut self,
        app: &mut Application<Model>,
        state: &mut Model,
        event: &MouseEvent,
    ) {
        let position = Point::new(0f32, 0f32);
        let size = *self.root.size();
        self.root
            .mouse_down(event, &Properties { position, size }, app, state)
    }

    pub fn mouse_up(
        &mut self,
        app: &mut Application<Model>,
        state: &mut Model,
        event: &MouseEvent,
    ) {
        self.root.mouse_up(event, app, state)
    }

    pub fn double_click(&mut self, state: &mut Model, event: &MouseEvent) {}

    pub fn mouse_drag(&mut self, state: &mut Model, event: &MouseEvent) {
        let properties = Properties {
            size: *self.root.size(),
            position: *self.root.position(),
        };
        self.root.mouse_dragged(event, &properties, state);
    }

    pub fn mouse_moved(&mut self, state: &mut Model, event: &MouseEvent) {
        self.root.mouse_moved(event, state);
    }

    pub fn mouse_leave(&mut self, state: &mut Model, event: &MouseEvent) {}
    pub fn keyboard_event(&mut self, state: &mut Model, event: &KeyboardInput) {
        self.root.keyboard_event(event, state);
    }
    pub fn character_received(&mut self, state: &mut Model, character: char) {
        self.root.character_received(character, state);
    }
    pub fn layout(&mut self, constraints: &BoxConstraints, state: &Model) {
        let size = self.root.layout(constraints, state);
        self.root.set_size(&size);
    }

    pub fn layout_child_with_name(&mut self, child_name: &str, state: &Model) {
        // self.root.layout_child_with_name(child_name, state)
    }

    pub fn paint(&mut self, state: &Model, canvas: &mut dyn Canvas2D) {
        canvas.clear(&self.style_ctx.theme(&self.theme).unwrap().background);
        self.root.paint(
            self.style_ctx.theme(&self.theme).unwrap(),
            canvas,
            self.root.size(),
            state,
        );
    }
}
