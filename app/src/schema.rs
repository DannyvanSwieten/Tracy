use crate::application::ServerApplication;

use super::application::Model;
use super::load_scene;
use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, Request, Response, Result, Variables,
};
use futures::lock::Mutex;
use serde_json::Value;
use std::sync::Arc;

pub struct Mesh {
    name: String,
    vertex_count: usize,
}

#[Object]
impl Mesh {
    async fn name(&self, _context: &Context<'_>) -> Result<&String> {
        Ok(&self.name)
    }

    async fn vertex_count(&self, _context: &Context<'_>) -> Result<usize> {
        Ok(self.vertex_count)
    }
}

pub struct Resources {
    meshes: Vec<Mesh>,
}

#[Object]
impl Resources {
    async fn meshes(&self, _context: &Context<'_>) -> Result<&Vec<Mesh>> {
        Ok(&self.meshes)
    }
}

pub struct Node {
    name: String,
    children: Vec<usize>,
    camera: Option<usize>,
    mesh: Option<usize>,
}

#[Object]
impl Node {
    async fn name(&self, _context: &Context<'_>) -> Result<&String> {
        Ok(&self.name)
    }

    async fn camera(&self, _context: &Context<'_>) -> Result<&Option<usize>> {
        Ok(&self.camera)
    }

    async fn mesh(&self, _context: &Context<'_>) -> Result<&Option<usize>> {
        Ok(&self.mesh)
    }

    async fn children(&self, _context: &Context<'_>) -> Result<&Vec<usize>> {
        Ok(&self.children)
    }
}

pub struct Scene {
    name: String,
    nodes: Vec<Node>,
    root: usize,
}

#[Object]
impl Scene {
    async fn name(&self, _context: &Context<'_>) -> Result<&String> {
        Ok(&self.name)
    }

    async fn root(&self, _context: &Context<'_>) -> Result<usize> {
        Ok(self.root)
    }

    async fn nodes(&self, _context: &Context<'_>) -> Result<&'_ Vec<Node>> {
        Ok(&self.nodes)
    }
}

pub struct Query;

#[Object]
impl Query {
    async fn scenes(&self, context: &Context<'_>) -> Result<Vec<Scene>> {
        let model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        let scenes = model
            .scenes
            .iter()
            .map(|scene| Scene {
                nodes: scene
                    .nodes
                    .iter()
                    .map(|node| Node {
                        name: node.name.clone(),
                        children: node.children.clone(),
                        camera: node.camera,
                        mesh: node.mesh,
                    })
                    .collect(),
                name: scene.name.clone(),
                root: scene.root,
            })
            .collect();

        Ok(scenes)
    }

    async fn resources(&self, context: &Context<'_>) -> Result<Resources> {
        let model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        Ok(Resources {
            meshes: model.scenes[model.current_scene]
                .geometry_buffer_views()
                .iter()
                .map(|view| Mesh {
                    name: view.name.clone(),
                    vertex_count: view.vertex_count() as usize,
                })
                .collect(),
        })
    }

    async fn width(&self, context: &Context<'_>) -> Result<u32> {
        let model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        Ok(model.renderer.output_width)
    }

    async fn height(&self, context: &Context<'_>) -> Result<u32> {
        let model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        Ok(model.renderer.output_height)
    }

    async fn build(&self, context: &Context<'_>) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.build_current_scene();
        Ok(true)
    }

    async fn set_active_scene(&self, context: &Context<'_>, scene_id: usize) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.current_scene = scene_id;
        Ok(true)
    }

    async fn render(&self, context: &Context<'_>, batches: u32) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.render(batches);
        Ok(true)
    }

    async fn image(&self, context: &Context<'_>) -> Result<Vec<u8>> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        Ok(model.download_image())
    }

    async fn move_camera(&self, context: &Context<'_>, x: f32, y: f32, z: f32) -> Result<Vec<f32>> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.renderer.move_camera(&nalgebra_glm::vec3(x, y, z));
        let new_position = model.renderer.camera_position();
        Ok(vec![new_position[0], new_position[1], new_position[2]])
    }

    async fn set_camera_position(
        &self,
        context: &Context<'_>,
        x: f32,
        y: f32,
        z: f32,
    ) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model
            .renderer
            .set_camera_position(&nalgebra_glm::vec3(x, y, z));
        Ok(true)
    }

    async fn look_at(&self, context: &Context<'_>, x: f32, y: f32, z: f32) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.renderer.look_at(&nalgebra_glm::vec3(x, y, z));
        Ok(true)
    }

    async fn load(&self, context: &Context<'_>, path: String) -> Result<usize> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        match load_scene(&path) {
            Ok(scene) => {
                model.scenes.push(scene);
                Ok(model.scenes.len() - 1)
            }
            Err(_) => Ok(usize::MAX),
        }
    }

    async fn create_floor(&self, context: &Context<'_>, y: f32) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        let current_scene = model.current_scene;
        model.scenes[current_scene].create_floor(y);
        Ok(true)
    }
}

pub type Schema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

pub fn new_schema(renderer: Arc<Mutex<Model>>) -> Schema {
    Schema::build(Query, EmptyMutation::default(), EmptySubscription)
        .data(renderer)
        .finish()
}
