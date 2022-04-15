use std::sync::Arc;

use crate::{
    mesh_resource::MeshResource,
    resource::{GpuResource, Resource},
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
    mesh: Option<Arc<Resource<MeshResource>>>,
    children: Vec<usize>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            local_transform: Mat4x4::new_nonuniform_scaling(&vec3(1.0, 1.0, 1.0)),
            mesh: None,
            children: Vec::new(),
        }
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

    pub fn with_transform(&mut self, transform: Mat4x4) -> &mut Self {
        self.local_transform = transform;
        self
    }

    pub fn allocate_resources(
        &self,
        parent_transform: Mat4x4,
        scene_graph: &SceneGraph,
        resources: &Resources,
        device: &DeviceContext,
        rtx: &RtxContext,
        mut scene: Scene,
    ) -> Scene {
        let this_transform = parent_transform * self.local_transform;
        // if let Some(mesh) = &self.mesh {
        //     let m = &cpu_cache.mesh(mesh.id).mesh;
        //     let id = gpu_cache.add_mesh(device, rtx, m, mesh.id);
        //     scene.create_instance(id, &this_transform, cpu_cache.material(m.material).material);
        //     scene.add_mesh(id);

        //     let material = cpu_cache.material(m.material);
        //     for i in 0..4 {
        //         if material.material.maps[i] != -1 {
        //             let image = cpu_cache.texture(material.material.maps[i] as usize);
        //             gpu_cache.add_texture(device, &image.image, image.id);
        //         }
        //     }
        // }

        // for child in &self.children {
        //     scene = scene_graph.node(*child).allocate_resources(
        //         this_transform,
        //         scene_graph,
        //         cpu_cache,
        //         gpu_cache,
        //         device,
        //         rtx,
        //         scene,
        //     );
        // }
        scene
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
                .with_mesh(resources.get_unchecked(*primitive));
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
        &self,
        gpu_cache: &mut GpuResourceCache,
        parent_transform: Mat4x4,
        device: &DeviceContext,
        rtx: &RtxContext,
    ) -> Scene {
        let mut scene = Scene::default();
        for node in &self.nodes {
            if let Some(mesh) = &node.mesh {
                let gpu_mat = gpu_cache.add_material(device, rtx, &mesh.material);
                let gpu_mesh = gpu_cache.add_mesh(device, rtx, mesh);
                let mut shape = Shape::new(gpu_mesh);
                shape.create_instance(
                    gpu_mat,
                    &vec3(0., 0., 0.),
                    &vec3(1., 1., 1.),
                    &vec3(0., 0., 0.),
                );

                scene.attach_shape(Arc::new(shape))
            }
        }

        scene
    }
}

unsafe impl Send for SceneGraph {}
