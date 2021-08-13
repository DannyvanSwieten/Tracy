extern crate nalgebra_glm as glm;

use legion::*;
use renderer::geometry::Vertex;
use renderer::{renderer::Renderer, scene::Scene};
use std::rc::Rc;
use vk_utils::{device_context::DeviceContext, gpu::Gpu, swapchain::Swapchain};

struct Transform {
    position: glm::Vec3,
    scale: glm::Vec3,
    orientation: glm::Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glm::Vec3::default(),
            scale: glm::vec3(1., 1., 1.),
            orientation: glm::Quat::default(),
        }
    }
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = glm::vec3(x, y, z);
        self
    }

    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = glm::vec3(x, y, z);
        self
    }
}

#[derive(Default)]
struct StaticMesh {
    geometry_id: usize,
    instance_id: usize,
}

#[system(for_each)]
fn scene_builder(transform: &Transform, mesh: &StaticMesh, #[resource] scene: &mut Scene) {
    scene.set_position(
        mesh.instance_id,
        transform.position.x,
        transform.position.y,
        transform.position.z,
    );

    scene.set_scale(
        mesh.instance_id,
        transform.scale.x,
        transform.scale.y,
        transform.scale.z,
    );
}

#[system]
fn acquire(#[resource] swapchain: &mut Swapchain, #[resource] index: &mut FrameIndex) {
    let (_succes, i, framebuffer, semaphore) = swapchain
        .next_frame_buffer()
        .expect("Acquire next framebuffer failed");
    index.i = i;
    index.framebuffer = framebuffer;
    index.semaphore = semaphore;
}

#[system]
fn present(
    #[resource] device: &DeviceContext,
    #[resource] swapchain: &Swapchain,
    #[resource] image_view: &ash::vk::ImageView,
    #[resource] index: &FrameIndex,
) {
    device.graphics_queue().as_ref().unwrap().begin_render_pass(
        swapchain.render_pass(),
        &index.framebuffer,
        swapchain.physical_width(),
        swapchain.physical_height(),
        |cmd| {
            //image_renderer.render(device, &cmd, image_view, 0);
            cmd
        },
    );
    swapchain.swap(
        device.graphics_queue().as_ref().unwrap().vk_queue(),
        &index.semaphore,
        index.i,
    );
}

#[derive(Default)]
struct FrameIndex {
    pub i: u32,
    pub framebuffer: ash::vk::Framebuffer,
    pub semaphore: ash::vk::Semaphore,
}

pub struct Game {
    pub device: Rc<DeviceContext>,
    world: World,
    pub renderer: Renderer,
    resources: Resources,
    schedule: Schedule,
    iteration: u32,
    pub output_image: Option<ash::vk::Image>,
}

impl Game {
    pub fn new(gpu: &Gpu, device: Rc<DeviceContext>) -> Self {
        let mut world = legion::World::default();
        let mut renderer = Renderer::new(gpu, &device, 1920, 1080);
        let mut scene = renderer::scene::Scene::new();
        let (document, buffers, _) = gltf::import(
            "C:\\Users\\danny\\Documents\\code\\tracey\\assets\\Cube\\glTF\\Cube.gltf",
        )
        .unwrap();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                let vertices: Vec<Vertex> = reader
                    .read_positions()
                    .unwrap()
                    .map(|vertex_position| {
                        Vertex::new(vertex_position[0], vertex_position[1], vertex_position[2])
                    })
                    .collect();

                let indices: Vec<u32> = if let Some(iter) = reader.read_indices() {
                    iter.into_u32().collect()
                } else {
                    (0..vertices.len() as u32).collect()
                };

                let geometry_id = scene.add_geometry(indices, vertices);
                let instance_id = scene.create_instance(geometry_id);
                world.push((
                    Transform::default()
                        .with_position(0., -2., 0.)
                        .with_scale(50., 0.1, 50.),
                    StaticMesh {
                        geometry_id,
                        instance_id,
                    },
                ));

                for i in 0..10 {
                    let instance_id = scene.create_instance(geometry_id);
                    world.push((
                        Transform::default()
                            .with_position(-5. + i as f32, 0., -3.)
                            .with_scale(0.4, 0.4, 0.4),
                        StaticMesh {
                            geometry_id,
                            instance_id,
                        },
                    ));
                }
            }
        }

        renderer.build(&device, &scene);

        let mut resources = Resources::default();
        resources.insert(scene);

        let this = Self {
            device,
            world,
            resources,
            renderer,
            schedule: Schedule::builder()
                .add_system(scene_builder_system())
                .build(),
            iteration: 0,
            output_image: None,
        };

        this
    }

    pub fn resources(&mut self) -> &mut Resources {
        &mut self.resources
    }

    pub fn tick(&mut self) {
        self.schedule.execute(&mut self.world, &mut self.resources);
        self.renderer
            .build(&self.device, &*self.resources.get::<Scene>().unwrap());
        let (image, _) = self.renderer.render(&self.device);
        self.output_image = Some(image);
    }
}
