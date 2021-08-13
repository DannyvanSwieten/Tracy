use crate::game::Game;
use ui::{
    canvas_2d::Canvas2D, node::Node, user_interface::UIDelegate, widget::StyleSheet, widget::*,
    window_event::MouseEvent, window_event::MouseEventType,
};
pub struct MyState {
    pub count: u32,
    pub game: Option<Game>,
}

use skia_safe::Rect;

struct ViewPortWidget {}

use nalgebra_glm::Vec3;

impl Widget<MyState> for ViewPortWidget {
    fn paint(
        &mut self,
        state: &MyState,
        rect: &Rect,
        canvas: &mut dyn Canvas2D,
        _style: &StyleSheet,
    ) {
        if let Some(game) = &state.game {
            if let Some(image) = game.output_image {
                canvas.draw_vk_image_rect(&Rect::from_wh(rect.width(), rect.height()), rect, &image)
            }
        }
    }

    fn resized(&mut self, state: &mut MyState, rect: &Rect) {
        if let Some(game) = &mut state.game {
            game.renderer
                .resize(&game.device, rect.width() as u32, rect.height() as u32);

            game.output_image = None;
        }
    }

    fn mouse_drag(&mut self, state: &mut MyState, _: &Rect, event: &MouseEvent) {
        if let Some(game) = &mut state.game {
            game.renderer.move_camera(&Vec3::new(
                event.delta_position().x * 0.025,
                0.,
                event.delta_position().y * 0.025,
            ));
        }
    }
}

fn build_top_bar() -> Node<MyState> {
    Node::new("div")
}

fn build_left_side_bar() -> Node<MyState> {
    Node::new("div")
}

fn build_view_port() -> Node<MyState> {
    Node::new_with_widget("viewport", Box::new(ViewPortWidget {}))
        .with_relative_max_constraints(Some(70.), None)
}

fn build_right_side_bar() -> Node<MyState> {
    Node::new("div")
}

fn build_middle() -> Node<MyState> {
    Node::new_with_widget("body", Box::new(Stack::new(Orientation::Horizontal)))
        .with_rebuild_callback(|_| {
            Some(vec![
                build_left_side_bar(),
                build_view_port(),
                build_right_side_bar(),
            ])
        })
        .with_relative_max_constraints(None, Some(60.))
        .with_spacing(5.)
}

fn build_bottom() -> Node<MyState> {
    Node::new("div")
}

pub struct MyUIDelegate {}
impl UIDelegate<MyState> for MyUIDelegate {
    fn build(&self, _: &str, _: &MyState) -> Node<MyState> {
        Node::new("body").with_widget(Container::new()).with_child(
            Node::new("body")
                .with_widget(Stack::new(Orientation::Vertical))
                .with_rebuild_callback(|_| {
                    Some(vec![build_top_bar(), build_middle(), build_bottom()])
                })
                .with_spacing(5.),
        )
    }
}
