use crate::game::Game;
use legion::*;
use ui::{
    canvas_2d::Canvas2D, node::Node, user_interface::UIDelegate, widget::StyleSheet, widget::*,
    window_event::MouseEvent, window_event::MouseEventType,
};

pub struct EditorState {
    selected_entity: Option<Entity>,
    entities: Vec<Entity>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            selected_entity: None,
            entities: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity, name: &str) {
        self.entities.push(entity);
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        //self.entities.remove(&entity);
    }

    pub fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    pub fn get_entity_by_index(&self, index: usize) -> Option<Entity> {
        if index < self.entities.len() {
            Some(self.entities[index])
        } else {
            None
        }
    }

    pub fn select_entity(&mut self, index: usize) {
        self.selected_entity = self.get_entity_by_index(index)
    }
}

pub struct GameEditor {
    pub editor_state: EditorState,
    pub game: Option<Game>,
    pub playing: bool,
}

impl GameEditor {
    pub fn new() -> Self {
        Self {
            editor_state: EditorState::new(),
            game: None,
            playing: false,
        }
    }

    pub fn play(&mut self) {
        //todo serialize gamestate
        self.playing = true
    }

    pub fn stop(&mut self) {
        //todo deserialize old gamestate
        self.playing = false
    }

    pub fn create_entity(&mut self, name: &str) {
        if let Some(game) = &mut self.game {
            let e = game.create_entity();
            self.editor_state.add_entity(e, name);
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        if let Some(game) = &mut self.game {
            self.editor_state.remove_entity(entity);
            game.remove_entity(entity)
        } else {
            false
        }
    }

    pub fn tick(&mut self) {
        if let Some(game) = &mut self.game {
            if self.playing {
                game.tick()
            }
        }
    }
}

use skia_safe::Rect;

struct ViewPortWidget {}

use nalgebra_glm::Vec3;

impl Widget<GameEditor> for ViewPortWidget {
    fn paint(
        &mut self,
        state: &GameEditor,
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

    fn resized(&mut self, state: &mut GameEditor, rect: &Rect) {
        if let Some(game) = &mut state.game {
            game.renderer
                .resize(&game.device, rect.width() as u32, rect.height() as u32);

            game.output_image = None;
        }
    }

    fn mouse_drag(&mut self, state: &mut GameEditor, _: &Rect, event: &MouseEvent) {
        if let Some(game) = &mut state.game {
            game.renderer.move_camera(&Vec3::new(
                event.delta_position().x * 0.25,
                0.,
                event.delta_position().y * 0.25,
            ));
        }
    }
}

struct EntityTableDelegate {}

impl TableDelegate<GameEditor> for EntityTableDelegate {
    fn row_selected(&mut self, index: usize, state: &mut GameEditor) -> Action<GameEditor> {
        if let Some(_) = &state.game {
            state.editor_state.select_entity(index)
        }
        Action::Layout {
            nodes: vec!["root"],
        }
    }
    fn row_count(&self, state: &GameEditor) -> usize {
        if let Some(game) = &state.game {
            game.world.len()
        } else {
            0
        }
    }
}

fn build_top_bar() -> Node<GameEditor> {
    Node::new("div").with_mouse_event_callback(MouseEventType::MouseUp, |event, state| {
        let menu = ui::widget::PopupMenu::new(0, "root").with_item(1, "New Entity");
        let request =
            ui::widget::PopupRequest::new(menu, |menu, submenu, state: &mut GameEditor| {
                state.create_entity("New Entity");
                ui::widget::Action::Layout {
                    nodes: vec!["root"],
                }
            });

        ui::widget::Action::PopupRequest {
            request: request,
            position: *event.global_position(),
        }
    })
}

fn build_left_side_bar() -> Node<GameEditor> {
    Node::new("div").with_child(
        Node::new("table").with_widget(Table::<GameEditor>::new(EntityTableDelegate {})),
    )
}

fn build_view_port() -> Node<GameEditor> {
    Node::new_with_widget("viewport", Box::new(ViewPortWidget {}))
        .with_relative_max_constraints(Some(70.), None)
}

fn build_right_side_bar() -> Node<GameEditor> {
    Node::new("div")
}

fn build_middle() -> Node<GameEditor> {
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

fn build_bottom() -> Node<GameEditor> {
    Node::new("div")
}

pub struct MyUIDelegate {}
impl UIDelegate<GameEditor> for MyUIDelegate {
    fn build(&self, _: &str, _: &GameEditor) -> Node<GameEditor> {
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
