use std::rc::Rc;

use crate::{gpu_scene::Mesh, instance::Instance};

pub struct Shape {
    uid: usize,
    mesh: Rc<Mesh>,
    instances: Vec<Instance>,
}
