use crate::image_renderer::ImageRenderer;
use legion::*;
use renderer::{renderer::Renderer, scene::Scene};
use vk_utils::{device_context::DeviceContext, swapchain::Swapchain};

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
fn scene_uploader(
    #[resource] device: &DeviceContext,
    #[resource] scene: &Scene,
    #[resource] renderer: &mut Renderer,
) {
    renderer.build(device, scene);
}

#[system]
fn render(
    #[resource] device: &mut DeviceContext,
    #[resource] renderer: &mut Renderer,
    #[resource] image_view: &mut ash::vk::ImageView,
) {
    renderer.set_camera(&glm::vec3(0., 0., 10.), &glm::vec3(0., 0., 0.));
    *image_view = *renderer.render(device);
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
    #[resource] image_renderer: &ImageRenderer,
    #[resource] image_view: &ash::vk::ImageView,
    #[resource] index: &FrameIndex,
) {
    device.graphics_queue().as_ref().unwrap().begin_render_pass(
        swapchain.render_pass(),
        &index.framebuffer,
        swapchain.physical_width(),
        swapchain.physical_height(),
        |cmd| {
            image_renderer.render(device, &cmd, image_view, 0);
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
    world: World,
    resources: Resources,
    schedule: Schedule,
    iteration: u32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::default(),
            resources: Resources::default(),
            schedule: Schedule::builder()
                .add_system(acquire_system())
                .add_system(scene_builder_system())
                .add_system(scene_uploader_system())
                .add_system(render_system())
                .add_system(present_system())
                .build(),
            iteration: 0,
        }
    }

    pub fn resources(&mut self) -> &mut Resources {
        &mut self.resources
    }

    pub fn tick(&mut self) {
        self.schedule.execute(&mut self.world, &mut self.resources)
    }
}

// use ash::vk::{
//     PhysicalDeviceAccelerationStructureFeaturesKHR, PhysicalDeviceFeatures2KHR,
//     PhysicalDeviceRayTracingPipelineFeaturesKHR, PhysicalDeviceVulkan12Features,
// };

// use ash::version::InstanceV1_1;

// pub mod game;
// use game::Game;

// struct Delegate {
//     pub game: Game,
// }

// impl ApplicationDelegate<MyState> for Delegate {
//     fn application_will_update(
//         &mut self,
//         _app: &Application<MyState>,
//         _state: &mut MyState,
//         _window_registry: &mut WindowRegistry<MyState>,
//         _target: &EventLoopWindowTarget<()>,
//     ) {
//         let start = std::time::Instant::now();
//         self.game.tick();
//         println!("{} fps", 1000 as f64 / start.elapsed().as_millis() as f64);
//     }
//     fn application_will_start(
//         &mut self,
//         app: &Application<MyState>,
//         _state: &mut MyState,
//         window_registry: &mut WindowRegistry<MyState>,
//         target: &EventLoopWindowTarget<()>,
//     ) {
//         // let mut scene = renderer::scene::Scene::new();
//         // let (document, buffers, _) = gltf::import(
//         //     "C:\\Users\\danny\\Documents\\code\\tracey\\assets\\Cube\\glTF\\Cube.gltf",
//         // )
//         // .unwrap();

//         // for mesh in document.meshes() {
//         //     for primitive in mesh.primitives() {
//         //         let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
//         //         let vertices: Vec<Vertex> = reader
//         //             .read_positions()
//         //             .unwrap()
//         //             .map(|vertex_position| {
//         //                 Vertex::new(vertex_position[0], vertex_position[1], vertex_position[2])
//         //             })
//         //             .collect();

//         //         let indices: Vec<u32> = if let Some(iter) = reader.read_indices() {
//         //             iter.into_u32().collect()
//         //         } else {
//         //             (0..vertices.len() as u32).collect()
//         //         };

//         //         let geometry_id = scene.add_geometry(indices, vertices);
//         //         let instance_id = scene.create_instance(geometry_id);
//         //         self.world.push((
//         //             Transform::default()
//         //                 .with_position(0., -2., 0.)
//         //                 .with_scale(50., 0.1, 50.),
//         //             StaticMesh {
//         //                 geometry_id,
//         //                 instance_id,
//         //             },
//         //         ));

//         //         for i in 0..10 {
//         //             let instance_id = scene.create_instance(geometry_id);
//         //             self.world.push((
//         //                 Transform::default()
//         //                     .with_position(-5. + i as f32, 0., -3.)
//         //                     .with_scale(0.4, 0.4, 0.4),
//         //                 StaticMesh {
//         //                     geometry_id,
//         //                     instance_id,
//         //                 },
//         //             ));
//         //         }
//         //     }
//         // }

//         //     let gpu = &app
//         //         .vulkan()
//         //         .hardware_devices_with_queue_support(renderer::vk::QueueFlags::GRAPHICS)[0];
//         //     let device = unsafe {
//         //         let mut rt_features =
//         //             PhysicalDeviceRayTracingPipelineFeaturesKHR::builder().ray_tracing_pipeline(true);
//         //         let mut address_features =
//         //             PhysicalDeviceVulkan12Features::builder().buffer_device_address(true);
//         //         let mut acc_features = PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
//         //             .acceleration_structure(true);
//         //         let mut features2 = PhysicalDeviceFeatures2KHR::default();
//         //         gpu.vulkan()
//         //             .vk_instance()
//         //             .get_physical_device_features2(*gpu.vk_physical_device(), &mut features2);

//         //         gpu.device_context(
//         //             &[
//         //                 ash::extensions::khr::Swapchain::name(),
//         //                 ash::extensions::khr::RayTracingPipeline::name(),
//         //                 ash::extensions::khr::AccelerationStructure::name(),
//         //                 ash::extensions::khr::DeferredHostOperations::name(),
//         //                 ash::extensions::khr::ExternalMemoryFd::name(),
//         //             ],
//         //             |builder| {
//         //                 builder
//         //                     .push_next(&mut address_features)
//         //                     .push_next(&mut rt_features)
//         //                     .push_next(&mut acc_features)
//         //                     .enabled_features(&features2.features)
//         //             },
//         //         )
//         //     };

//         //     let window = window_registry.create_window(target, "Application Title", 1200, 800);
//         //     let surface = unsafe {
//         //         ash_window::create_surface(
//         //             app.vulkan().library(),
//         //             app.vulkan().vk_instance(),
//         //             &window,
//         //             None,
//         //         )
//         //         .expect("Surface creation failed")
//         //     };
//         //     let swapchain = Swapchain::new(
//         //         app.vulkan(),
//         //         &gpu,
//         //         &device,
//         //         &surface,
//         //         None,
//         //         0,
//         //         window.inner_size().width,
//         //         window.inner_size().height,
//         //     );

//         //     let canvas = SkiaGpuCanvas2D::new(
//         //         app.vulkan(),
//         //         &gpu,
//         //         &device,
//         //         3,
//         //         window.inner_size().width,
//         //         window.inner_size().height,
//         //     );
//         //     let ui = Box::new(UIWindowDelegate::new(
//         //         Box::new(canvas),
//         //         Box::new(MyUIDelegate {}),
//         //     ));

//         //     let renderer = Renderer::new(
//         //         &gpu,
//         //         &device,
//         //         swapchain.physical_width(),
//         //         swapchain.physical_height(),
//         //     );

//         //     let image_renderer = ImageRenderer::new(
//         //         &device,
//         //         swapchain.render_pass(),
//         //         swapchain.image_count() as u32,
//         //         swapchain.physical_width(),
//         //         swapchain.physical_height(),
//         //     );

//         //     self.game.resources().insert(device);
//         //     self.game.resources().insert(scene);
//         //     self.game.resources().insert(renderer);
//         //     self.game.resources().insert(swapchain);
//         //     self.game.resources().insert(FrameIndex::default());
//         //     self.game.resources().insert(ash::vk::ImageView::null());
//         //     self.game.resources().insert(image_renderer);

//         //     window_registry.register_with_delegate(window, ui);
//     }
// }
