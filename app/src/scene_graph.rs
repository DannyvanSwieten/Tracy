use crate::resources::CpuResourceCache;

use super::instancer::Instancer;
use nalgebra_glm::Mat3x4;
use renderer::{
    context::RtxContext,
    gpu_scene::{GpuResourceCache, GpuScene},
};
use vk_utils::device_context::DeviceContext;

pub struct DefaultInstancer {}

impl Instancer for DefaultInstancer {
    fn instance(&self) {
        todo!()
    }
}

pub struct Mesh {
    id: usize,
    instancer: Box<dyn Instancer>,
}

impl Mesh {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            instancer: Box::new(DefaultInstancer {}),
        }
    }
}

pub struct Actor {
    transform: Mat3x4,
    mesh: Option<Mesh>,
    children: Vec<Actor>,
}

impl Actor {
    pub fn new() -> Self {
        Self {
            transform: Mat3x4::default(),
            mesh: None,
            children: Vec::new(),
        }
    }

    pub fn with_mesh(mut self, id: usize) -> Self {
        self.mesh = Some(Mesh::new(id));
        self
    }

    pub fn allocate_resources(
        &self,
        cpu_cache: &CpuResourceCache,
        gpu_cache: &mut GpuResourceCache,
        device: &DeviceContext,
        rtx: &RtxContext,
        mut scene: GpuScene,
    ) -> GpuScene {
        if let Some(mesh) = &self.mesh {
            let id = gpu_cache.add_mesh(device, rtx, &cpu_cache.meshes[mesh.id].mesh, mesh.id);
            scene.add_mesh(id);
        }

        for child in &self.children {
            scene = child.allocate_resources(cpu_cache, gpu_cache, device, rtx, scene);
        }
        scene
    }
}

pub struct SceneGraph {
    root: Actor,
}

impl SceneGraph {
    pub fn new(root: Actor) -> Self {
        Self { root }
    }
}

impl SceneGraph {
    pub fn build(
        &self,
        cpu_cache: &CpuResourceCache,
        gpu_cache: &mut GpuResourceCache,
        device: &DeviceContext,
        rtx: &RtxContext,
    ) -> GpuScene {
        let scene = GpuScene::new();
        self.root
            .allocate_resources(cpu_cache, gpu_cache, device, rtx, scene)
    }
}
