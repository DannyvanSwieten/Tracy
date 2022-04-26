use crate::scene_graph::{Entity, SceneGraph};

use super::application::Model;
use super::load_scene_gltf;
use async_graphql::{Context, Object, Result, Subscription};
use futures::lock::Mutex;
use futures::stream::Stream;
use std::sync::Arc;
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct Mesh {
    pub name: String,
    pub material: Material,
}

#[Object]
impl Mesh {
    async fn name(&self, _context: &Context<'_>) -> Result<&String> {
        Ok(&self.name)
    }

    async fn material(&self, _context: &Context<'_>) -> Result<&Material> {
        Ok(&self.material)
    }
}

#[derive(Clone)]
pub struct Node {
    id: usize,
    name: String,
    children: Vec<usize>,
    camera: Option<usize>,
    mesh: Option<Mesh>,
}

impl Node {
    pub fn new(id: usize, scene: &SceneGraph, actor: &Entity) -> Self {
        let mesh = if let Some(meshes) = actor.mesh() {
            Some(Mesh {
                name: "Untitled".to_string(),
                material: Material::new(
                    &"Untitled".to_string(),
                    &Material {
                        name: "".to_string(),
                    },
                ),
            })
        } else {
            None
        };

        Self {
            id,
            name: "".to_string(),
            children: actor.children().to_vec(),
            camera: None,
            mesh,
        }
    }
}

#[Object]
impl Node {
    async fn name(&self, _context: &Context<'_>) -> Result<&String> {
        Ok(&self.name)
    }

    async fn camera(&self, _context: &Context<'_>) -> Result<&Option<usize>> {
        Ok(&self.camera)
    }

    async fn meshes(&self, _context: &Context<'_>) -> Result<&Option<Mesh>> {
        Ok(&self.mesh)
    }

    async fn children(&self, _context: &Context<'_>) -> Result<&Vec<usize>> {
        Ok(&self.children)
    }

    async fn id(&self, _context: &Context<'_>) -> Result<usize> {
        Ok(self.id)
    }
}

#[derive(Clone)]
pub struct Material {
    name: String,
    //mat: gpu_scene::Material,
}

impl Material {
    pub fn new(name: &String, scene_material: &Material) -> Self {
        Self {
            name: name.clone(),
            //mat: scene_material.clone(),
        }
    }
}

#[Object]
impl Material {
    async fn name(&self, _context: &Context<'_>) -> Result<&String> {
        Ok(&self.name)
    }

    async fn roughness(&self, _context: &Context<'_>) -> Result<f32> {
        Ok(1.)
    }

    async fn metalness(&self, _context: &Context<'_>) -> Result<f32> {
        Ok(0.)
    }

    async fn base_color(&self, _context: &Context<'_>) -> Result<[f32; 4]> {
        Ok([
            // self.mat.base_color[0],
            // self.mat.base_color[1],
            // self.mat.base_color[2],
            // self.mat.base_color[3],
            0., 0., 0., 0.,
        ])
    }
}

#[derive(Clone)]
pub struct Scene {
    name: String,
    materials: Vec<Material>,
    meshes: Vec<Mesh>,
    nodes: Vec<Node>,
    root: usize,
}

impl Scene {
    pub fn new(scene: &SceneGraph) -> Self {
        Self {
            name: scene.name().to_string(),
            materials: Vec::new(),
            nodes: scene
                .nodes()
                .iter()
                .enumerate()
                .map(|(id, node)| Node::new(id, scene, node))
                .collect(),
            meshes: Vec::new(),
            root: scene.root(),
        }
    }
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

    async fn meshes(&self, _context: &Context<'_>) -> Result<&'_ Vec<Mesh>> {
        Ok(&self.meshes)
    }

    async fn materials(&self, _context: &Context<'_>) -> Result<&'_ Vec<Material>> {
        Ok(&self.materials)
    }
}

pub struct Sub {}

#[Subscription]
impl Sub {
    async fn scene_loaded(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = Scene>> {
        let model = ctx.data::<Arc<Mutex<Model>>>()?.lock().await;
        let rx = model.broadcasters.scene_loaded_broadcaster.subscribe();
        let stream =
            tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| result.ok());
        Ok(stream)
    }
}

pub struct Query;

#[Object]
impl Query {
    async fn scenes(&self, context: &Context<'_>) -> Result<Scene> {
        let model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        let scene = Scene::new(&model.project.scene_graph);
        Ok(scene)
    }

    async fn width(&self, context: &Context<'_>) -> Result<u32> {
        let model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        Ok(model.renderer.output_width)
    }

    async fn height(&self, context: &Context<'_>) -> Result<u32> {
        let model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        Ok(model.renderer.output_height)
    }

    async fn image(&self, context: &Context<'_>) -> Result<Vec<u8>> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        Ok(model.download_image())
    }

    async fn get_node(
        &self,
        context: &Context<'_>,
        scene_id: usize,
        node_id: usize,
    ) -> Result<Node> {
        let model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        let n = model.project.scene_graph.node(node_id);
        Ok(Node::new(node_id, &model.project.scene_graph, n))
    }
}

pub struct Mutation {}

#[Object]
impl Mutation {
    async fn look_at(&self, context: &Context<'_>, x: f32, y: f32, z: f32) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.renderer.set_camera_target(x, y, z);
        Ok(true)
    }

    async fn move_camera(
        &self,
        context: &Context<'_>,
        dx: f32,
        dy: f32,
        dz: f32,
    ) -> Result<Vec<f32>> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.renderer.move_camera_position(dx, dy, dz);
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
        model.renderer.set_camera_position(x, y, z);
        Ok(true)
    }

    async fn load(&self, context: &Context<'_>, path: String) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        match load_scene_gltf(&path, &mut model.cpu_resource_cache) {
            Ok(mut scenes) => {
                model.project.scene_graph = scenes.pop().unwrap();
                let scene_data = Scene::new(&model.project.scene_graph);

                match model.broadcasters.scene_loaded_broadcaster.send(scene_data) {
                    Ok(_) => (),
                    Err(error) => println!("{}", error.to_string()),
                }
                Ok(true)
            }
            Err(e) => Err(async_graphql::Error::new(e.to_string())),
        }
    }

    async fn build(&self, context: &Context<'_>) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        Ok(model.build_scene())
    }

    async fn new_project(&self, context: &Context<'_>, name: String) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.new_project(&name);
        Ok(true)
    }

    async fn import(&self, context: &Context<'_>, path: String) -> Result<Option<usize>> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        Ok(model.project.import(&path))
    }

    async fn render(&self, context: &Context<'_>, batches: u32) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.render(batches);
        Ok(true)
    }
}

pub type Schema = async_graphql::Schema<Query, Mutation, Sub>;

pub fn new_schema(model: Arc<Mutex<Model>>) -> Schema {
    Schema::build(Query, Mutation {}, Sub {})
        .data(model)
        .finish()
}