use super::instancer::Instancer;
use nalgebra_glm::Mat3x4;

pub struct Mesh {
    id: usize,
    instancer: Box<dyn Instancer>,
}

pub struct Actor {
    transform: Mat3x4,
    mesh: Option<Mesh>,
    children: Vec<Actor>,
}

pub struct SceneGraph {
    root: Actor,
}

impl SceneGraph {
    pub fn build() {}
}
