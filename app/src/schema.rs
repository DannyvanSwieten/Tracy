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
    meshes: Option<Vec<Mesh>>,
}

impl Node {
    pub fn new(
        id: usize,
        scene: &renderer::scene::Scene,
        scene_node: &renderer::scene::SceneGraphNode,
    ) -> Self {
        let meshes = if let Some(meshes) = &scene_node.mesh {
            Some(
                meshes
                    .iter()
                    .map(|mesh| {
                        let mat = scene.materials()[*mesh];
                        Mesh {
                            name: scene.geometry_buffer_views()[*mesh].name.clone(),
                            material: Material::new(&scene.material_names[*mesh], &mat),
                        }
                    })
                    .collect(),
            )
        } else {
            None
        };

        Self {
            id,
            name: scene_node.name.clone(),
            children: scene_node.children.clone(),
            camera: None,
            meshes,
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

    async fn meshes(&self, _context: &Context<'_>) -> Result<&Option<Vec<Mesh>>> {
        Ok(&self.meshes)
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
    mat: renderer::scene::Material,
}

impl Material {
    pub fn new(name: &String, scene_material: &renderer::scene::Material) -> Self {
        Self {
            name: name.clone(),
            mat: scene_material.clone(),
        }
    }
}

#[Object]
impl Material {
    async fn name(&self, _context: &Context<'_>) -> Result<&String> {
        Ok(&self.name)
    }

    async fn roughness(&self, _context: &Context<'_>) -> Result<f32> {
        Ok(self.mat.metallic_roughness[1])
    }

    async fn metalness(&self, _context: &Context<'_>) -> Result<f32> {
        Ok(self.mat.metallic_roughness[0])
    }

    async fn base_color(&self, _context: &Context<'_>) -> Result<[f32; 4]> {
        Ok([
            self.mat.color[0],
            self.mat.color[1],
            self.mat.color[2],
            self.mat.color[3],
        ])
    }
}

#[derive(Clone)]
pub struct Scene {
    id: usize,
    name: String,
    materials: Vec<Material>,
    meshes: Vec<Mesh>,
    nodes: Vec<Node>,
    root: usize,
}

impl Scene {
    pub fn new(id: usize, scene: &renderer::scene::Scene) -> Self {
        Self {
            id,
            name: scene.name.clone(),
            materials: scene
                .materials
                .iter()
                .enumerate()
                .map(|(index, m)| Material::new(&scene.material_names[index], m))
                .collect(),
            nodes: scene
                .nodes
                .iter()
                .enumerate()
                .map(|(id, node)| Node::new(id, scene, node))
                .collect(),
            meshes: Vec::new(),
            root: scene.root,
        }
    }
}

#[Object]
impl Scene {
    async fn id(&self, _context: &Context<'_>) -> Result<usize> {
        Ok(self.id)
    }

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
    async fn scenes(&self, context: &Context<'_>) -> Result<Vec<Scene>> {
        let model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        let scenes = model
            .scenes
            .iter()
            .enumerate()
            .map(|(id, scene)| Scene::new(id, scene))
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
        if scene_id < model.scenes.len() {
            let n = &model.scenes[scene_id].nodes[node_id];
            Ok(Node::new(node_id, &model.scenes[scene_id], n))
        } else {
            Err(async_graphql::Error::new("Nope"))
        }
    }
}

pub struct Mutation {}

#[Object]
impl Mutation {
    async fn look_at(&self, context: &Context<'_>, x: f32, y: f32, z: f32) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.renderer.look_at(&nalgebra_glm::vec3(x, y, z));
        Ok(true)
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

    async fn load(&self, context: &Context<'_>, path: String) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        match load_scene_gltf(&path) {
            Ok(scene) => {
                model.scenes.push(scene);
                let scene = model.scenes.last().unwrap();
                let scene_data = Scene::new(model.scenes.len() - 1, scene);

                match model.broadcasters.scene_loaded_broadcaster.send(scene_data) {
                    Ok(_) => (),
                    Err(error) => println!("{}", error.to_string()),
                }
                model.current_scene = Some(model.scenes.len() - 1);
                Ok(true)
            }
            Err(e) => Err(async_graphql::Error::new(e.to_string())),
        }
    }

    async fn create_floor(&self, context: &Context<'_>, y: f32) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        if let Some(current_scene) = model.current_scene {
            model.scenes[current_scene].create_floor(y);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn build(&self, context: &Context<'_>, scene_id: usize) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        Ok(model.build_scene(scene_id))
    }

    async fn set_active_scene(&self, context: &Context<'_>, scene_id: usize) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        model.current_scene = Some(scene_id);
        Ok(true)
    }

    async fn render(&self, context: &Context<'_>, scene_id: usize, batches: u32) -> Result<bool> {
        let mut model = context.data::<Arc<Mutex<Model>>>()?.lock().await;
        if scene_id < model.scenes.len() {
            model.render(scene_id, batches);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub type Schema = async_graphql::Schema<Query, Mutation, Sub>;

pub fn new_schema(model: Arc<Mutex<Model>>) -> Schema {
    Schema::build(Query, Mutation {}, Sub {})
        .data(model)
        .finish()
}
