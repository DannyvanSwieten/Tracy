use crate::scene_graph::SceneGraph;

pub struct Project {
    pub name: String,
    pub scene_graph: SceneGraph,
}

impl Project {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            scene_graph: SceneGraph::new(name),
        }
    }
}
