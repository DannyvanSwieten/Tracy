pub mod game;
pub mod user_interface;

use ash::vk::{
    PhysicalDeviceAccelerationStructureFeaturesKHR, PhysicalDeviceFeatures2KHR,
    PhysicalDeviceRayTracingPipelineFeaturesKHR, PhysicalDeviceVulkan12Features,
};

use user_interface::{GameEditor, MyUIDelegate};

use ui::application::Application;
use ui::ui_application_delegate::UIApplicationDelegate;

fn main() {
    let app: Application<GameEditor> = Application::new("My Application");
    let app_delegate = UIApplicationDelegate::new()
        .on_update(|_, state: &mut GameEditor| {
            if let Some(game) = &mut state.game {
                game.tick()
            }
        })
        .with_window("My Window", 1920, 1080, MyUIDelegate {})
        .on_device_created(|gpu, device, state| state.game = Some(game::Game::new(gpu, device)))
        .with_device_builder(|gpu, mut extensions| {
            extensions.push(ash::extensions::khr::RayTracingPipeline::name());
            extensions.push(ash::extensions::khr::AccelerationStructure::name());
            extensions.push(ash::extensions::khr::DeferredHostOperations::name());

            let mut rt_features =
                PhysicalDeviceRayTracingPipelineFeaturesKHR::builder().ray_tracing_pipeline(true);
            let mut address_features =
                PhysicalDeviceVulkan12Features::builder().buffer_device_address(true);
            let mut acc_features = PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
                .acceleration_structure(true);
            let mut features2 = PhysicalDeviceFeatures2KHR::default();
            unsafe {
                gpu.vulkan()
                    .vk_instance()
                    .get_physical_device_features2(*gpu.vk_physical_device(), &mut features2);
            }

            gpu.device_context(&extensions, |builder| {
                builder
                    .push_next(&mut address_features)
                    .push_next(&mut rt_features)
                    .push_next(&mut acc_features)
                    .enabled_features(&features2.features)
            })
        });

    app.run(Box::new(app_delegate), GameEditor::new());
}
