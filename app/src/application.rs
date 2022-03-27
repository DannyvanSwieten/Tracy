use super::schema;
use super::server::Server;
use futures::lock::Mutex;
use renderer::cpu_scene::Scene;
use renderer::renderer::Renderer;
use std::sync::Arc;
use vk_utils::device_context::DeviceContext;

pub struct Broadcasters {
    pub scene_loaded_broadcaster: tokio::sync::broadcast::Sender<schema::Scene>,
}

impl Broadcasters {
    pub fn new() -> Self {
        Self {
            scene_loaded_broadcaster: tokio::sync::broadcast::channel(16).0,
        }
    }
}
pub struct Model {
    pub device: DeviceContext,
    pub renderer: Renderer,
    pub scenes: Vec<Scene>,
    pub current_scene: Option<usize>,
    pub has_new_frame: bool,
    pub broadcasters: Broadcasters,
}

impl Model {
    pub fn new(device: DeviceContext, renderer: Renderer) -> Self {
        Self {
            device,
            renderer,
            scenes: vec![],
            current_scene: None,
            has_new_frame: false,
            broadcasters: Broadcasters::new(),
        }
    }

    pub fn build_scene(&mut self, scene_id: usize) -> bool {
        if scene_id < self.scenes.len() {
            //self.renderer.build(&self.device, &self.scenes[scene_id]);
            self.renderer.clear();
            true
        } else {
            false
        }
    }

    pub fn render(&mut self, scene_id: usize, batches: u32) {
        for _ in 0..batches {
            //self.renderer.render(2, &self.device);
        }
        self.has_new_frame = true;
    }

    pub fn download_image(&mut self) -> Vec<u8> {
        let buffer = self.renderer.download_image(&self.device);
        self.has_new_frame = false;
        buffer.copy_data::<u8>()
    }
}

pub struct ServerApplication {
    pub model: super::ServerContext,
    pub server: Server,
}

impl ServerApplication {
    pub fn new(device: DeviceContext, address: &str, width: u32, height: u32) -> Self {
        let renderer = Renderer::new(&device, width, height);
        let model = Arc::new(Mutex::new(Model::new(device, renderer)));
        Self {
            model: model.clone(),
            server: Server::new(model.clone(), address, true),
        }
    }
}
