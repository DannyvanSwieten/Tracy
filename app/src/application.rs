use super::server::Server;
use futures::lock::Mutex;
use renderer::renderer::Renderer;
use renderer::scene::Scene;
use std::sync::Arc;
use vk_utils::device_context::DeviceContext;
pub struct Model {
    pub device: DeviceContext,
    pub renderer: Renderer,
    pub scenes: Vec<Scene>,
    pub current_scene: usize,
    pub has_new_frame: bool,
}

impl Model {
    pub fn new(device: DeviceContext, renderer: Renderer) -> Self {
        Self {
            device,
            renderer,
            scenes: vec![Scene::default()],
            current_scene: 0,
            has_new_frame: false,
        }
    }

    pub fn build_current_scene(&mut self) {
        self.renderer
            .build(&self.device, &self.scenes[self.current_scene]);
        self.renderer.clear();
    }

    pub fn render(&mut self, batches: u32) {
        for _ in 0..batches {
            self.renderer.render(2, &self.device);
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
    pub fn new(device: DeviceContext, address: &str) -> Self {
        let renderer = Renderer::new(&device, 1270, 720);
        let model = Arc::new(Mutex::new(Model::new(device, renderer)));
        Self {
            model: model.clone(),
            server: Server::new(model.clone(), address, true),
        }
    }

    pub fn build_current_scene(&mut self) {
        if let Some(mut model) = self.model.try_lock() {
            model.build_current_scene();
        }
    }
}
