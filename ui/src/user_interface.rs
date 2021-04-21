use crate::node::*;
use crate::widget::*;
use crate::window::MouseEvent;
use skia_safe::canvas::Canvas;
use skia_safe::Point;
use skia_safe::Rect;
use skia_safe::Size;

pub trait UIDelegate<AppState> {
    fn build(&self, section: &str, state: &AppState) -> Node<AppState>;
}

#[repr(C)]
pub struct UserInterface<DataModel> {
    pub root: Node<DataModel>,
    pub material: Material,
    pub hovered: u32,

    actions: Vec<Action<DataModel>>,
    pop_up: Option<Node<DataModel>>,
    pop_up_request: Option<PopupRequest<DataModel>>,
}

impl<DataModel: 'static> UserInterface<DataModel> {
    pub fn new(root: Node<DataModel>) -> Self {
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

    pub fn update(&mut self, state: &mut DataModel) {
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

    pub fn build_popup(&mut self, request: PopupRequest<DataModel>, position: &Point) {
        self.pop_up = Some(request.build());
        self.pop_up_request = Some(request);
        let node = self.pop_up.as_mut().unwrap();
        node.rect.left = position.x;
        node.rect.top = position.y;
    }

    pub fn resize(&mut self, state: &DataModel, width: u32, height: u32) {
        self.root.rect.set_wh(width as f32, height as f32);
        self.layout(state);
    }

    pub fn mouse_down(&mut self, state: &mut DataModel, event: &MouseEvent) {
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

    pub fn mouse_up(&mut self, state: &mut DataModel, window_id: usize, event: &MouseEvent) {
        if let Some(popup) = self.pop_up.as_mut() {
            self.actions.push(popup.mouse_up(state, window_id, event));
            return;
        }

        self.actions
            .push(self.root.mouse_up(state, window_id, event))
    }

    pub fn double_click(&mut self, state: &mut DataModel, event: &MouseEvent) {
        self.root.double_click(state, event);
    }

    pub fn mouse_drag(&mut self, state: &mut DataModel, event: &MouseEvent) {
        if let Some(popup) = self.pop_up.as_mut() {
            self.actions.push(popup.mouse_drag(state, event));
            return;
        }

        self.root.mouse_drag(state, event);
    }

    pub fn mouse_moved(&mut self, state: &mut DataModel, event: &MouseEvent) {
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

    pub fn mouse_leave(&mut self, state: &mut DataModel, event: &MouseEvent) {
        self.root.mouse_leave(state, event);
    }
    pub fn layout(&mut self, state: &DataModel) {
        self.root.layout(state);
    }

    pub fn layout_child_with_name(&mut self, child_name: &str, state: &mut DataModel) {
        self.root.layout_child_with_name(child_name, state)
    }

    pub fn paint(&mut self, state: &DataModel, canvas: &mut Canvas) {
        canvas.clear(skia_safe::Color::WHITE);
        self.root.draw(state, canvas, &self.material);
        if let Some(popup) = self.pop_up.as_mut() {
            popup.layout(state);
            popup.draw(state, canvas, &self.material);
        }
    }

    // pub fn paint_gpu(&mut self, state: &mut DataModel, ctx: &mut GraphicsContext) {
    //     self.root.draw_gpu(state, ctx);
    // }
}
