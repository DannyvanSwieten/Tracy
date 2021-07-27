use ui::{node::Node, user_interface::UIDelegate, widget::*, window_event::MouseEventType};

pub struct MyState {
    pub count: u32,
}

pub struct MyUIDelegate {}
impl UIDelegate<MyState> for MyUIDelegate {
    fn build(&self, _: &str, _: &MyState) -> Node<MyState> {
        Node::new("body")
            .with_widget(Container::new())
            .with_padding(25.)
            .with_child(
                Node::new("div")
                    .with_name("root")
                    .with_widget(Stack::new(Orientation::Horizontal))
                    .with_rebuild_callback(|state| {
                        Some(std::vec![
                            Node::<MyState>::new("btn")
                                .with_widget(Button::new("Up"))
                                .with_event_callback(MouseEventType::MouseUp, |_event, state| {
                                    state.count = state.count + 1;
                                    Action::Layout {
                                        nodes: vec!["root"],
                                    }
                                }),
                            Node::<MyState>::new("btn")
                                .with_widget(Button::new("Reset"))
                                .with_event_callback(MouseEventType::MouseUp, |_event, state| {
                                    state.count = 0;
                                    Action::Layout {
                                        nodes: vec!["root"],
                                    }
                                }),
                            Node::new("btn").with_widget(Label::new(
                                &(String::from("Count: ") + &state.count.to_string())
                            )),
                        ])
                    })
                    .with_padding(25.)
                    .with_spacing(5.),
            )
    }
}
