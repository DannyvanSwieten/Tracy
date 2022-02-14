extern crate nalgebra_glm as glm;
use legion::*;
use rand::Rng;
use rapier3d::prelude::*;
use renderer::geometry::Vertex;
use renderer::{renderer::Renderer, scene::Scene};
use std::rc::Rc;
use vk_utils::{device_context::DeviceContext, gpu::Gpu, swapchain::Swapchain};

pub struct Physics {
    gravity: nalgebra::Vector3<Real>,
    body_set: RigidBodySet,
    collider_set: ColliderSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    joint_set: JointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),
}

impl Physics {
    pub fn new() -> Self {
        Self {
            gravity: vector![0.0, -9.81, 0.0],
            body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            joint_set: JointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    pub fn tick(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.body_set,
            &mut self.collider_set,
            &mut self.joint_set,
            &mut self.ccd_solver,
            &self.physics_hooks,
            &self.event_handler,
        )
    }

    pub fn insert_rigid_body(&mut self, body: RigidBody) -> RigidBodyHandle {
        self.body_set.insert(body)
    }
}

unsafe impl Send for Physics {}
unsafe impl Sync for Physics {}

struct TransformComponent {
    position: glm::Vec3,
    scale: glm::Vec3,
    orientation: glm::Quat,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: glm::Vec3::default(),
            scale: glm::vec3(1., 1., 1.),
            orientation: glm::Quat::identity(),
        }
    }
}

impl TransformComponent {
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

struct RigidBodyComponent {
    pub body_handle: RigidBodyHandle,
}

impl RigidBodyComponent {
    pub fn new(body_handle: RigidBodyHandle) -> Self {
        Self { body_handle }
    }
}

#[derive(Default)]
struct StaticMesh {
    geometry_id: usize,
    instance_id: usize,
}

#[system(for_each)]
fn scene_builder(
    body: Option<&RigidBodyComponent>,
    transform: &mut TransformComponent,
    mesh: &StaticMesh,
    #[resource] scene: &mut Scene,
    #[resource] physics: &Physics,
) {
    if let Some(rb) = &body {
        let b = &physics.body_set[rb.body_handle];
        transform.position.x = b.translation().x;
        transform.position.y = b.translation().y;
        transform.position.z = b.translation().z;

        if let Some(_) = b.rotation().axis() {
            let scaled = b.rotation().scaled_axis();
            transform.orientation.coords[0] = scaled.x;
            transform.orientation.coords[1] = scaled.y;
            transform.orientation.coords[2] = scaled.z;
        } else {
            transform.orientation.coords[0] = 0.;
            transform.orientation.coords[1] = 0.;
            transform.orientation.coords[2] = 0.;
        }

        transform.orientation.coords[3] = 1.;
    }

    let id = glm::Mat4x4::identity();
    let s = glm::scaling(&transform.scale);
    let t = glm::translation(&transform.position);
    let r = glm::quat_to_mat4(&glm::quat_normalize(&transform.orientation));

    let result = t * r * s * id;
    let m43 = glm::make_mat4x3(glm::transpose(&result).as_slice());
    scene.set_transform(mesh.instance_id, &m43);
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
    pub world: World,
    pub renderer: Renderer,
    resources: Resources,
    schedule: Schedule,
    iteration: u32,
    pub output_image: Option<ash::vk::Image>,
    instant: std::time::Instant,
}

impl Game {
    pub fn new(gpu: &Gpu, device: Rc<DeviceContext>) -> Self {
        let mut world = legion::World::default();
        let mut physics = Physics::new();
        let renderer = Renderer::new(gpu, &device, 1920, 1080);
        let mut scene = renderer::scene::Scene::new();
        let dir = std::env::current_exe()
            .expect("current dir check failed")
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("assets");
        let (document, buffers, _) = gltf::import(
            dir.join("Cube")
                .join("gltf")
                .join("Cube.gltf")
                .to_str()
                .unwrap(),
        )
        .unwrap();

        let restitution: f32 = 0.25;

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

                let geometry_id = scene.add_geometry(indices.clone(), vertices.clone());
                let instance_id = scene.create_instance(geometry_id);
                let collider = ColliderBuilder::cuboid(100.0, 0.5, 100.0).build();
                physics.collider_set.insert(collider);
                world.push((
                    TransformComponent::default()
                        .with_position(0., -0.5, 0.)
                        .with_scale(100., 0.5, 100.),
                    StaticMesh {
                        geometry_id,
                        instance_id,
                    },
                ));
            }
        }

        let mut resources = Resources::default();
        resources.insert(scene);
        resources.insert(physics);

        Self {
            device,
            world,
            resources,
            renderer,
            schedule: Schedule::builder()
                .add_system(scene_builder_system())
                .build(),
            iteration: 0,
            output_image: None,
            instant: std::time::Instant::now(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        self.world.push((TransformComponent::new(),))
    }

    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        self.world.remove(entity)
    }

    pub fn resources(&mut self) -> &mut Resources {
        &mut self.resources
    }

    pub fn import_gltf(&mut self, file: &std::path::PathBuf) {
        let scene = &mut *self.resources.get_mut::<Scene>().unwrap();
        match gltf::import(file.to_str().unwrap()) {
            Ok((document, buffers, _)) => {
                for mesh in document.meshes() {
                    for primitive in mesh.primitives() {
                        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                        let vertices: Vec<Vertex> = reader
                            .read_positions()
                            .unwrap()
                            .map(|vertex_position| {
                                Vertex::new(
                                    vertex_position[0],
                                    vertex_position[1],
                                    vertex_position[2],
                                )
                            })
                            .collect();
                        let indices: Vec<u32> = if let Some(iter) = reader.read_indices() {
                            iter.into_u32().collect()
                        } else {
                            (0..vertices.len() as u32).collect()
                        };
                        let geometry_id = scene.add_geometry(indices.clone(), vertices.clone());
                        let instance_id = scene.create_instance(geometry_id);
                        self.world.push((
                            TransformComponent::default(),
                            StaticMesh {
                                geometry_id,
                                instance_id,
                            },
                        ));
                    }
                }
            }
            Err(_) => {}
        }
    }

    pub fn tick(&mut self) {
        {
            self.iteration += 1;
            let op = self.resources.get_mut::<Physics>();
            if let Some(mut physics) = op {
                //(*physics).integration_parameters.dt = 1. / frames_per_second as f32;
                (*physics).tick()
            }
        }

        self.schedule.execute(&mut self.world, &mut self.resources);
        self.renderer
            .build(&self.device, &*self.resources.get::<Scene>().unwrap());
        let (image, _) = self.renderer.render(&self.device);
        self.output_image = Some(image);

        self.instant = std::time::Instant::now();
    }
}
