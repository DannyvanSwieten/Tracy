use crate::canvas3d::Canvas3D;
use crate::node::*;
use crate::widget::*;
use crate::window_event::MouseEvent;
use skia_safe::canvas::Canvas;
use skia_safe::Point;

pub trait UIDelegate<AppState> {
    fn build(&self, section: &str, state: &AppState) -> Node<AppState>;
}

#[repr(C)]
pub struct UserInterface<AppState> {
    pub root: Node<AppState>,
    pub material: Material,
    pub hovered: u32,

    actions: Vec<Action<AppState>>,
    pop_up: Option<Node<AppState>>,
    pop_up_request: Option<PopupRequest<AppState>>,
}

impl<AppState: 'static> UserInterface<AppState> {
    pub fn new(root: Node<AppState>) -> Self {
        let ui = UserInterface {
            root,
            material: Material::new(),
            hovered: 0,
            actions: Vec::new(),
            pop_up: None,
            pop_up_request: None,
        };

        ui
    }

    pub fn update(&mut self, state: &mut AppState) {
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

    pub fn build_popup(&mut self, request: PopupRequest<AppState>, position: &Point) {
        self.pop_up = Some(request.build());
        self.pop_up_request = Some(request);
        let node = self.pop_up.as_mut().unwrap();
        node.rect.left = position.x;
        node.rect.top = position.y;
    }

    pub fn resize(&mut self, state: &AppState, width: u32, height: u32) {
        self.root.rect.set_wh(width as f32, height as f32);
        self.layout(state);
    }

    pub fn mouse_down(&mut self, state: &mut AppState, event: &MouseEvent) {
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

    pub fn mouse_up(&mut self, state: &mut AppState, event: &MouseEvent) {
        if let Some(popup) = self.pop_up.as_mut() {
            self.actions.push(popup.mouse_up(state, event));
            return;
        }

        self.actions.push(self.root.mouse_up(state, event))
    }

    pub fn double_click(&mut self, state: &mut AppState, event: &MouseEvent) {
        self.root.double_click(state, event);
    }

    pub fn mouse_drag(&mut self, state: &mut AppState, event: &MouseEvent) {
        if let Some(popup) = self.pop_up.as_mut() {
            self.actions.push(popup.mouse_drag(state, event));
            return;
        }

        self.root.mouse_drag(state, event);
    }

    pub fn mouse_moved(&mut self, state: &mut AppState, event: &MouseEvent) {
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

    pub fn mouse_leave(&mut self, state: &mut AppState, event: &MouseEvent) {
        self.root.mouse_leave(state, event);
    }
    pub fn layout(&mut self, state: &AppState) {
        self.root.layout(state);
    }

    pub fn layout_child_with_name(&mut self, child_name: &str, state: &mut AppState) {
        self.root.layout_child_with_name(child_name, state)
    }

    pub fn paint(&mut self, state: &AppState, canvas: &mut Canvas) {
        canvas.clear(
            *self
                .material
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

    pub fn paint_3d(&mut self, state: &AppState, _canvas_3d: &mut dyn Canvas3D) {
        self.root.draw_3d(state);
    }
}
