use image::save_buffer;
use renderer::geometry::*;
use renderer::renderer::*;
use renderer::scene::*;
use ui::application::{Application, ApplicationDelegate, WindowRegistry};
use ui::ui_window::UIWindowDelegate;
use user_interface::MyUIDelegate;
use winit::event_loop::EventLoopWindowTarget;
pub mod user_interface;
use legion::*;
use user_interface::MyState;
extern crate nalgebra_glm as glm;

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
struct Velocity {
    dx: f32,
    dy: f32,
    dz: f32,
}

#[derive(Default)]
struct StaticMesh {
    geometry_id: usize,
    instance_id: usize,
}

struct Delegate {
    world: World,
    resources: Resources,
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
fn scene_uploader(#[resource] scene: &mut Scene, #[resource] renderer: &mut Renderer) {
    renderer.build(scene)
}

#[system]
fn render(#[resource] renderer: &mut Renderer) {
    renderer.set_camera(&glm::vec3(0., 0., 5.), &glm::vec3(0., 0., 0.));
    renderer.render();
    let output = renderer.download_image().copy_data::<u8>();
    save_buffer("image.png", &output, 1200, 800, image::ColorType::Rgba8)
        .expect("Image write failed");

    println!("Render!");
}

impl ApplicationDelegate<MyState> for Delegate {
    fn application_will_update(
        &mut self,
        _app: &Application<MyState>,
        _state: &mut MyState,
        _window_registry: &mut WindowRegistry<MyState>,
        _target: &EventLoopWindowTarget<()>,
    ) {
        let mut schedule = Schedule::builder()
            .add_system(scene_builder_system())
            .add_system(scene_uploader_system())
            .add_system(render_system())
            .build();

        schedule.execute(&mut self.world, &mut self.resources)
    }
    fn application_will_start(
        &mut self,
        app: &Application<MyState>,
        state: &mut MyState,
        window_registry: &mut WindowRegistry<MyState>,
        target: &EventLoopWindowTarget<()>,
    ) {
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
                self.world.push((
                    Transform::default()
                        .with_position(0., 2., 0.)
                        .with_scale(100., 0.1, 100.),
                    Velocity::default(),
                    StaticMesh {
                        geometry_id,
                        instance_id,
                    },
                ));

                let instance_id = scene.create_instance(geometry_id);
                self.world.push((
                    Transform::default(),
                    Velocity::default(),
                    StaticMesh {
                        geometry_id,
                        instance_id,
                    },
                ));
            }
        }

        let gpu = &app
            .vulkan()
            .hardware_devices_with_queue_support(renderer::vk::QueueFlags::GRAPHICS)[0];
        let mut renderer = Renderer::new(&gpu);

        renderer.initialize(1200, 800);

        self.resources.insert(scene);
        self.resources.insert(renderer);

        let window = window_registry.create_window(target, "Application Title", 1000, 200);

        let ui = match UIWindowDelegate::<MyState>::new(
            app,
            state,
            &window,
            Box::new(MyUIDelegate {}),
        ) {
            Ok(ui_window_delegate) => Box::new(ui_window_delegate),
            Err(message) => panic!("{}", message),
        };
        window_registry.register(window, ui);
    }
}

fn main() {
    let app: Application<MyState> = Application::new("My Application");
    app.run(
        Box::new(Delegate {
            world: World::default(),
            resources: Resources::default(),
        }),
        MyState { count: 0 },
    );
}
