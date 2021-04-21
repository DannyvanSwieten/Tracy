extern crate ash;
extern crate ash_window;
extern crate winit;

use ash::version::InstanceV1_0;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::path::PathBuf;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
};

use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain, Win32Surface},
    mvk::MacOSSurface,
};

#[cfg(target_os = "macos")]
fn surface_extension_name() -> &'static CStr {
    MacOSSurface::name()
}

#[cfg(target_os = "windows")]
fn surface_extension_name() -> &'static CStr {
    Win32Surface::name()
}

pub use ash::version::{DeviceV1_0, EntryV1_0};
use ash::{vk, Device, Entry, Instance};
use vk::Queue;

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}

pub trait ApplicationDelegate<AppState> {
    fn application_will_start(
        &mut self,
        _: &Application<AppState>,
        _: &mut AppState,
        _: &EventLoopWindowTarget<()>,
    ) {
    }
    fn application_will_quit(
        &mut self,
        _: &mut Application<AppState>,
        _: &EventLoopWindowTarget<()>,
    ) {
    }

    fn close_button_pressed(&mut self, _: &winit::window::WindowId) -> ControlFlow {
        ControlFlow::Wait
    }
    fn window_destroyed(&mut self, _: &winit::window::WindowId) -> ControlFlow {
        ControlFlow::Wait
    }
    fn window_resized(
        &mut self,
        _: &winit::window::WindowId,
        _: &winit::dpi::PhysicalSize<u32>,
    ) -> ControlFlow {
        ControlFlow::Wait
    }

    fn window_moved(
        &mut self,
        _: &winit::window::WindowId,
        _: &winit::dpi::PhysicalPosition<i32>,
    ) -> ControlFlow {
        ControlFlow::Wait
    }

    fn window_got_focus(&mut self, _: &winit::window::WindowId) -> ControlFlow {
        ControlFlow::Wait
    }
    fn window_lost_focus(&mut self, _: &winit::window::WindowId) -> ControlFlow {
        ControlFlow::Wait
    }

    fn window_requested_redraw(
        &mut self,
        app: &Application<AppState>,
        state: &AppState,
        _: &winit::window::WindowId,
    ) -> ControlFlow {
        ControlFlow::Wait
    }

    fn file_dropped(&mut self, _: &winit::window::WindowId, _: &PathBuf) -> ControlFlow {
        ControlFlow::Wait
    }

    fn cursor_moved(
        &mut self,
        _: &winit::window::WindowId,
        _: &winit::dpi::PhysicalPosition<f64>,
    ) -> ControlFlow {
        ControlFlow::Wait
    }
}

pub struct Application<AppState> {
    debug_callback: vk::DebugUtilsMessengerEXT,
    vulkan_entry: Entry,
    vulkan_instance: Instance,
    primary_gpu: vk::PhysicalDevice,
    primary_device_context: Device,
    present_queue: Queue,
    present_queue_index: u32,
    _state: std::marker::PhantomData<AppState>,
}

impl<AppState: 'static> Application<AppState> {
    pub fn new(name: &str) -> Self {
        let app_name = CString::new(name).unwrap();
        let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layers_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let surface_extensions = vec![Surface::name(), surface_extension_name()];
        let mut extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        extension_names_raw.push(DebugUtils::name().as_ptr());

        let appinfo = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&app_name)
            .engine_version(0)
            .api_version(vk::make_version(1, 2, 0));

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names_raw);

        unsafe {
            let entry = Entry::new().unwrap();
            let instance: Instance = entry
                .create_instance(&create_info, None)
                .expect("Instance creation error");

            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
                .pfn_user_callback(Some(vulkan_debug_callback));

            let debug_utils_loader = DebugUtils::new(&entry, &instance);
            let debug_callback = debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap();

            //let surface = ash_window::create_surface(&entry, &instance, &window, None).unwrap();
            let pdevices = instance
                .enumerate_physical_devices()
                .expect("Physical device error");
            let surface_loader = Surface::new(&entry, &instance);
            let (primary_gpu, queue_family_index) = pdevices
                .iter()
                .map(|pdevice| {
                    instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, ref info)| {
                            let supports_graphic_and_surface =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                            if supports_graphic_and_surface {
                                Some((*pdevice, index))
                            } else {
                                None
                            }
                        })
                        .next()
                })
                .filter_map(|v| v)
                .next()
                .expect("Couldn't find suitable device.");
            let present_queue_index = queue_family_index as u32;
            let device_extension_names_raw = [Swapchain::name().as_ptr()];
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [1.0];

            let queue_info = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(present_queue_index)
                .queue_priorities(&priorities)
                .build()];

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_info)
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);

            let primary_device_context: Device = instance
                .create_device(primary_gpu, &device_create_info, None)
                .unwrap();

            let present_queue =
                primary_device_context.get_device_queue(present_queue_index as u32, 0);

            Self {
                debug_callback,
                vulkan_entry: entry,
                vulkan_instance: instance,
                primary_gpu,
                primary_device_context,
                present_queue,
                present_queue_index,
                _state: std::marker::PhantomData::<AppState>::default(),
            }
        }
    }

    pub fn vulkan_entry(&self) -> &Entry {
        &self.vulkan_entry
    }

    pub fn vulkan_instance(&self) -> &Instance {
        &self.vulkan_instance
    }

    pub fn primary_gpu(&self) -> &vk::PhysicalDevice {
        &self.primary_gpu
    }

    pub fn primary_device_context(&self) -> &Device {
        &self.primary_device_context
    }

    pub fn present_queue_and_index(&self) -> (&Queue, usize) {
        (&self.present_queue, self.present_queue_index as usize)
    }

    pub fn run(mut self, delegate: Box<dyn ApplicationDelegate<AppState>>, state: AppState) {
        let mut s = state;
        let event_loop = EventLoop::new();
        let mut d = delegate;

        d.application_will_start(&self, &mut s, &event_loop);

        event_loop.run(move |e, event_loop, control_flow| {
            *control_flow = ControlFlow::Wait;

            match e {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } => *control_flow = d.close_button_pressed(&window_id),

                Event::WindowEvent {
                    event: WindowEvent::Destroyed,
                    window_id,
                } => *control_flow = d.window_destroyed(&window_id),

                Event::WindowEvent {
                    event: WindowEvent::Moved(physical_position),
                    window_id,
                } => *control_flow = d.window_moved(&window_id, &physical_position),

                Event::WindowEvent {
                    event: WindowEvent::Resized(physical_size),
                    window_id,
                } => *control_flow = d.window_resized(&window_id, &physical_size),

                Event::WindowEvent {
                    event: WindowEvent::DroppedFile(path_buffer),
                    window_id,
                } => *control_flow = d.file_dropped(&window_id, &path_buffer),

                Event::WindowEvent {
                    event: WindowEvent::Focused(f),
                    window_id,
                } => {
                    *control_flow = if f {
                        d.window_got_focus(&window_id)
                    } else {
                        d.window_lost_focus(&window_id)
                    }
                }

                Event::RedrawRequested(window_id) => {
                    *control_flow = d.window_requested_redraw(&self, &s, &window_id)
                }

                Event::WindowEvent {
                    event:
                        WindowEvent::CursorMoved {
                            device_id,
                            position,
                            modifiers,
                        },
                    window_id,
                } => *control_flow = d.cursor_moved(&window_id, &position),
                _ => (),
            }

            match control_flow {
                ControlFlow::Exit => d.application_will_quit(&mut self, &event_loop),
                _ => (),
            }
        });
    }
}
