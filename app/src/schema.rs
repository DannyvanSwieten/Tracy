use super::application::Model;
use super::load_scene;
use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, Request, Response, Result, Variables,
};
use futures::lock::Mutex;
use serde_json::Value;
use std::sync::Arc;

pub struct Node {
    name: String,
}

#[Object]
impl Node {
    async fn name(&self, _context: &Context<'_>) -> Result<String> {
        Ok(self.name.clone())
    }
}

pub struct Scene {
    name: String,
    nodes: Vec<Node>,
}

#[Object]
impl Scene {
    async fn name(&self, _context: &Context<'_>) -> Result<String> {
        Ok(self.name.clone())
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
                    })
                    .collect(),
                name: scene.name.clone(),
            })
            .collect();

        Ok(scenes)
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
}

pub type Schema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

pub fn new_schema(renderer: Arc<Mutex<Model>>) -> Schema {
    Schema::build(Query, EmptyMutation::default(), EmptySubscription)
        .data(renderer)
        .finish()
}

// pub async fn execute(
//     composition: Arc<Mutex<CompositionHost>>,
//     query: &str,
//     args: Value,
// ) -> Response {
//     let schema = new_schema(composition);
//     let request = Request::new(query).variables(Variables::from_json(args));

//     schema.execute(request).await
// }
