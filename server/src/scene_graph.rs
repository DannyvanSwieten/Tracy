use std::sync::Arc;

use crate::{
    mesh_resource::MeshResource,
    resource::Resource,
    resources::{GpuResourceCache, Resources},
};

use super::instancer::Instancer;
use nalgebra_glm::{vec3, Mat4x4};
use renderer::{context::RtxContext, gpu_scene::Scene, shape::Shape};
use vk_utils::device_context::DeviceContext;

pub struct DefaultInstancer {}

impl Instancer for DefaultInstancer {
    fn instance(&self) {
        todo!()
    }
}

pub struct Entity {
    local_transform: Mat4x4,
    global_transform: Mat4x4,
    mesh: Option<Arc<Resource<MeshResource>>>,
    children: Vec<usize>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            local_transform: Mat4x4::new_nonuniform_scaling(&vec3(1.0, 1.0, 1.0)),
            global_transform: Mat4x4::new_nonuniform_scaling(&vec3(1.0, 1.0, 1.0)),
            mesh: None,
            children: Vec::new(),
        }
    }

    pub fn with_transform(&mut self, local_transform: Mat4x4) -> &mut Self {
        self.local_transform = local_transform;
        self
    }

    pub fn transform(&mut self, t: &Mat4x4) {
        self.global_transform = t * self.local_transform
    }

    pub fn with_position(&mut self, position: &[f32; 3]) -> &mut Self {
        self.local_transform.row_mut(3)[0] = position[0];
        self.local_transform.row_mut(3)[1] = position[1];
        self.local_transform.row_mut(3)[2] = position[2];

        self
    }

    pub fn with_orientation(&mut self, orientation: &[f32; 4]) -> &mut Self {
        let q = nalgebra_glm::quat(
            orientation[0],
            orientation[1],
            orientation[2],
            orientation[3],
        );
        let rotation_matrix = nalgebra_glm::quat_to_mat4(&q);
        self.local_transform = self.local_transform * rotation_matrix;
        self
    }

    pub fn with_scale(&mut self, scale: &[f32; 3]) -> &mut Self {
        self.local_transform = self
            .local_transform
            .append_nonuniform_scaling(&vec3(scale[0], scale[1], scale[2]));
        self
    }

    pub fn with_mesh(&mut self, mesh: Arc<Resource<MeshResource>>) -> &mut Self {
        self.mesh = Some(mesh);
        self
    }

    pub fn with_child(&mut self, actor: usize) -> &mut Self {
        self.children.push(actor);
        self
    }

    pub fn mesh(&self) -> &Option<Arc<Resource<MeshResource>>> {
        &self.mesh
    }

    pub fn children(&self) -> &[usize] {
        &self.children
    }
}

pub struct SceneGraph {
    name: String,
    root: usize,
    nodes: Vec<Entity>,
}

impl SceneGraph {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            root: 0,
            nodes: Vec::new(),
        }
    }

    pub fn transform(&mut self, t: &Mat4x4) {
        self.transform_node(self.root, t)
    }

    fn transform_node(&mut self, id: usize, t: &Mat4x4) {
        self.nodes[id].transform(t);
        let child_t = self.nodes[id].global_transform;
        for child in self.nodes[id].children.clone() {
            self.transform_node(child, &child_t);
        }
    }

    pub fn nodes_with_mesh_id(&self, id: usize) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| {
                if let Some(mesh) = &node.mesh {
                    if mesh.uid() == id {
                        Some(i)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn expand_node(&mut self, resources: &Resources, node_id: usize, primitives: &Vec<usize>) {
        self.nodes[node_id].mesh = None;
        for primitive in primitives {
            let child_id = self.create_node();
            self.nodes[node_id]
                .with_child(child_id)
                .with_mesh(resources.get_mesh_unchecked(*primitive));
        }
    }

    pub fn create_node(&mut self) -> usize {
        self.nodes.push(Entity::new());
        self.nodes.len() - 1
    }

    pub fn add_node(&mut self, actor: Entity) -> usize {
        self.nodes.push(actor);
        self.nodes.len() - 1
    }

    pub fn node(&self, id: usize) -> &Entity {
        &self.nodes[id]
    }

    pub fn node_mut(&mut self, id: usize) -> &mut Entity {
        &mut self.nodes[id]
    }

    pub fn nodes(&self) -> &[Entity] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut Vec<Entity> {
        &mut self.nodes
    }

    pub fn root(&self) -> usize {
        self.root
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl SceneGraph {
    pub fn build(
        &mut self,
        gpu_cache: &mut GpuResourceCache,
        parent_transform: Mat4x4,
        device: &DeviceContext,
        rtx: &RtxContext,
    ) -> Scene {
        self.transform(&parent_transform);
        let mut scene = Scene::default();
        for node in &self.nodes {
            if let Some(mesh) = &node.mesh {
                let gpu_mat = gpu_cache.add_material(device, rtx, &mesh.material);
                let gpu_mesh = gpu_cache.add_mesh(device, rtx, mesh);
                let mut shape = Shape::new(gpu_mesh);
                shape.create_instance(gpu_mat, &node.global_transform);

                scene.attach_shape(Arc::new(shape))
            }
        }

        scene
    }
}

unsafe impl Send for SceneGraph {}