use crate::game::Game;
use legion::*;
use ui::{
    canvas_2d::Canvas2D, node::Node, user_interface::UIDelegate, widget::StyleSheet, widget::*,
    window_event::MouseEvent, window_event::MouseEventType,
};

use skia_safe::Size;

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

    pub fn import_gltf(&mut self, file: &std::path::PathBuf) {
        if let Some(game) = &mut self.game {
            game.import_gltf(file)
        }
    }
}

use skia_safe::Rect;

struct ViewPort {}

use nalgebra_glm::Vec3;

impl Widget<GameEditor> for ViewPort {
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
    Node::new("div").with_name("top bar")
    // .with_mouse_event_callback(MouseEventType::MouseUp, |event, state| {
    //     let menu = ui::widget::PopupMenu::new(0, "root").with_item(1, "New Entity");
    //     let request =
    //         ui::widget::PopupRequest::new(menu, |menu, submenu, state: &mut GameEditor| {
    //             state.create_entity("New Entity");
    //             ui::widget::Action::Layout {
    //                 nodes: vec!["root"],
    //             }
    //         });

    //     ui::widget::Action::PopupRequest {
    //         request: request,
    //         position: *event.global_position(),
    //     }
    // })
}

fn build_left_side_bar() -> Node<GameEditor> {
    Node::new("div")
        .with_name("left bar")
        .child(Node::new("table").widget(Table::<GameEditor>::new(EntityTableDelegate {})))
}

fn build_view_port() -> Node<GameEditor> {
    Node::new("viewport")
        .with_name("Render viewport")
        .widget(ViewPort {})
        .with_flex(3.0)
        .on_file_drop(|state, file| {
            state.import_gltf(file);
            Action::None
        })
}

fn build_right_side_bar() -> Node<GameEditor> {
    Node::new("div").with_name("right bar")
}

fn build_middle() -> Node<GameEditor> {
    Node::new("body")
        .with_name("middle")
        .widget(HStack::new())
        .on_rebuild(|_| {
            Some(vec![
                build_left_side_bar(),
                build_view_port(),
                build_right_side_bar(),
            ])
        })
        .spacing(5.)
        .with_flex(2.)
}

fn build_bottom() -> Node<GameEditor> {
    Node::new("div").with_name("bttm")
}

pub struct MyUIDelegate {}
impl UIDelegate<GameEditor> for MyUIDelegate {
    fn build(&self, _: &str, _: &GameEditor) -> Node<GameEditor> {
        Node::new("body")
            .with_name("editor")
            .widget(Container::new())
            .child(
                Node::new("body")
                    .with_name("editor")
                    .widget(VStack::new())
                    .on_rebuild(|_| Some(vec![build_top_bar(), build_middle(), build_bottom()]))
                    .spacing(5.),
            )

        // Node::new("body").widget(Container::new()).child(
        //     Node::new("btn")
        //         .widget(Button::new("My Button"))
        //         .size(200.0, 75.0),
        // )
    }
}
