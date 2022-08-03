use crate::application_model::ApplicationModel;
use crate::canvas_2d::Canvas2D;
use crate::node::*;
use crate::widget::*;
use crate::window_event::MouseEvent;
use skia_safe::Point;

pub trait UIBuilder<Model: ApplicationModel> {
    fn build(&self, section: &str, state: &Model) -> Node<Model>;
    fn build_menu(&self, _state: Model) -> Option<PopupRequest<Model>> {
        None
    }

    fn menu_item_count(&self, _state: &Model) -> i32 {
        0
    }

    fn menu_item_name(&self, _state: &Model) -> &str {
        todo!()
    }
}

#[repr(C)]
pub struct UserInterface<Model: ApplicationModel> {
    pub root: Node<Model>,
    pub material: Material,
    pub hovered: u32,

    actions: Vec<Action<Model>>,
    pop_up: Option<Node<Model>>,
    pop_up_request: Option<PopupRequest<Model>>,
}

impl<Model: ApplicationModel + 'static> UserInterface<Model> {
    pub fn new(root: Node<Model>) -> Self {
        UserInterface {
            root,
            material: Material::new(),
            hovered: 0,
            actions: Vec::new(),
            pop_up: None,
            pop_up_request: None,
        }
    }

    pub fn file_dropped(&mut self, state: &mut Model, path: &std::path::PathBuf, position: &Point) {
        self.actions
            .push(self.root.file_dropped(state, path, position))
    }

    pub fn file_hovered(&mut self, state: &mut Model, path: &std::path::PathBuf, position: &Point) {
    }

    pub fn update(&mut self, state: &mut Model) {
        while let Some(a) = self.actions.pop() {
            match a {
                Action::None => (),
                Action::Layout { nodes } => {
                    for n in nodes {
                        self.layout_child_with_name(n, state)
                    }
                }
                Action::PopupRequest { request, position } => self.build_popup(request, &position),
                Action::TriggerPopupMenu { menu, sub_menu } => {
                    if let Some(request) = &mut self.pop_up_request {
                        self.actions.push((request.handler)(menu, sub_menu, state));
                    }

                    self.pop_up = None;
                    self.pop_up_request = None;
                }
            }
        }
    }

    fn build_popup(&mut self, request: PopupRequest<Model>, position: &Point) {
        self.pop_up = Some(request.build());
        self.pop_up_request = Some(request);
        let node = self.pop_up.as_mut().unwrap();
        node.rect.left = position.x;
        node.rect.top = position.y;
    }

    pub fn resize(&mut self, state: &Model, width: u32, height: u32) {
        self.root.rect.set_wh(width as f32, height as f32);
        self.root.build(state);
        self.root.calculate_size(&Constraints::new(
            width as f32,
            width as f32,
            height as f32,
            height as f32,
        ));
        self.layout(state);
    }

    pub fn resized(&mut self, state: &mut Model) {
        self.root.resized(state);
    }

    pub fn mouse_down(&mut self, state: &mut Model, event: &MouseEvent) {
        let mut dismiss_popup = false;
        if let Some(popup) = self.pop_up.as_mut() {
            if !popup.hit_test(&event.global_position()) {
                dismiss_popup = true;
            } else {
                self.actions.push(popup.mouse_down(state, event));
                return;
            }
        }

        if dismiss_popup {
            self.pop_up = None;
        }

        self.actions.push(self.root.mouse_down(state, event))
    }

    pub fn mouse_up(&mut self, state: &mut Model, event: &MouseEvent) {
        if let Some(popup) = self.pop_up.as_mut() {
            self.actions.push(popup.mouse_up(state, event));
            return;
        }

        self.actions.push(self.root.mouse_up(state, event))
    }

    pub fn double_click(&mut self, state: &mut Model, event: &MouseEvent) {
        self.root.double_click(state, event);
    }

    pub fn mouse_drag(&mut self, state: &mut Model, event: &MouseEvent) {
        if let Some(popup) = self.pop_up.as_mut() {
            self.actions.push(popup.mouse_drag(state, event));
            return;
        }

        self.root.mouse_drag(state, event);
    }

    pub fn mouse_moved(&mut self, state: &mut Model, event: &MouseEvent) {
        if let Some(popup) = self.pop_up.as_mut() {
            if let Some(uid) = popup.mouse_moved(state, event) {
                if self.hovered != uid {
                    if self.hovered != 0 {
                        popup.send_mouse_leave(state, self.hovered, event);
                    }
                    popup.send_mouse_enter(state, uid, event);
                }
                self.hovered = uid;
            }

            return;
        }

        if let Some(uid) = self.root.mouse_moved(state, event) {
            if self.hovered != uid {
                if self.hovered != 0 {
                    self.root.send_mouse_leave(state, self.hovered, event);
                }
                self.root.send_mouse_enter(state, uid, event);
            }
            self.hovered = uid;
        }
    }

    pub fn mouse_leave(&mut self, state: &mut Model, event: &MouseEvent) {
        self.root.mouse_leave(state, event);
    }
    pub fn layout(&mut self, state: &Model) {
        self.root.layout(state);
    }

    pub fn layout_child_with_name(&mut self, child_name: &str, state: &Model) {
        self.root.layout_child_with_name(child_name, state)
    }

    pub fn paint(&mut self, state: &Model, canvas: &mut dyn Canvas2D) {
        canvas.clear(
            self.material
                .get_child("body")
                .unwrap()
                .get("bg-color")
                .unwrap(),
        );
        self.root.draw(state, canvas, &self.material);
        if let Some(popup) = self.pop_up.as_mut() {
            popup.layout(state);
            popup.draw(state, canvas, &self.material);
        }
    }

    // pub fn paint_3d(&mut self, state: &Model, _canvas_3d: &mut dyn Canvas3D) {
    //     self.root.draw_3d(state);
    // }
}
