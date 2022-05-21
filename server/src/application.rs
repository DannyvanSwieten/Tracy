use crate::project::Project;
use crate::resources::{GpuResourceCache, Resources};

use super::schema;
use super::server::Server;
use futures::lock::Mutex;
use renderer::gpu_scene::{Frame, Scene};
use renderer::gpu_path_tracer::Renderer;
use std::rc::Rc;
use std::sync::Arc;
use vk_utils::device_context::DeviceContext;

pub struct Broadcasters {
    pub scene_loaded_broadcaster: tokio::sync::broadcast::Sender<schema::Scene>,
    pub new_project_created: tokio::sync::broadcast::Sender<bool>,
    pub node_added: tokio::sync::broadcast::Sender<schema::Node>,
}

impl Broadcasters {
    pub fn new() -> Self {
        Self {
            scene_loaded_broadcaster: tokio::sync::broadcast::channel(16).0,
            new_project_created: tokio::sync::broadcast::channel(16).0,
            node_added: tokio::sync::broadcast::channel(16).0,
        }
    }
}

pub struct Model {
    pub renderer: Renderer,
    pub cpu_resource_cache: Resources,
    pub gpu_resource_cache: GpuResourceCache,
    pub built_scenes: Vec<Scene>,
    pub cached_frames: Vec<Frame>,
    pub current_scene: Option<usize>,
    pub has_new_frame: bool,
    pub broadcasters: Broadcasters,
    pub project: Project,
}

impl Model {
    pub fn new(renderer: Renderer) -> Self {
        Self {
            renderer,
            cpu_resource_cache: Resources::default().with_default_material(),
            gpu_resource_cache: GpuResourceCache::default(),
            built_scenes: Vec::new(),
            cached_frames: Vec::new(),
            current_scene: None,
            has_new_frame: false,
            broadcasters: Broadcasters::new(),
            project: Project::tmp(),
        }
    }

    pub fn new_project(&mut self, name: &str) {
        self.project = Project::new(name).unwrap()
    }

    pub fn build_scene(&mut self) -> bool {
        let scene = self.project.scene_graph.build(
            &mut self.gpu_resource_cache,
            &self.cpu_resource_cache,
            nalgebra_glm::Mat4x4::identity(),
            self.renderer.device.clone(),
            &self.renderer.rtx,
        );
        let frame = self.renderer.build_frame(&scene);
        if self.cached_frames.len() == 0 {
            self.cached_frames.push(frame);
        } else {
            self.cached_frames[0] = frame;
        }

        self.renderer.clear();
        true
    }

    pub fn render(&mut self, batches: u32) {
        for _ in 0..batches {
            self.renderer.render_frame(&self.cached_frames[0], 1);
        }
        self.has_new_frame = true;
    }

    pub fn download_image<T>(&mut self) -> Vec<T>
    where
        T: Copy,
    {
        let buffer = self.renderer.download_image();
        self.has_new_frame = false;
        buffer.copy_data::<T>()
    }
}

pub struct ServerApplication {
    pub model: super::ServerContext,
    pub server: Server,
}

impl ServerApplication {
    pub fn new(device: Rc<DeviceContext>, address: &str, width: u32, height: u32) -> Self {
        let renderer = Renderer::new(device, width, height);
        let model = Arc::new(Mutex::new(Model::new(renderer)));
        Self {
            model: model.clone(),
            server: Server::new(model.clone(), address, true),
        }
    }
}

unsafe impl Send for Model {}
