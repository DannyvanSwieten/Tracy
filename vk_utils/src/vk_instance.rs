use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk::{
    make_version, ApplicationInfo, Bool32, DebugUtilsMessageSeverityFlagsEXT,
    DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT,
    DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT, InstanceCreateInfo, PhysicalDevice,
    QueueFlags, FALSE,
};
use ash::{Device, Entry, Instance};
use std::borrow::Cow;
use std::ffi::{CStr, CString};

use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Win32Surface},
    mvk::MacOSSurface,
};

use crate::gpu::Gpu;

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_type: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> Bool32 {
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

    FALSE
}

#[cfg(target_os = "macos")]
fn surface_extension_name() -> &'static CStr {
    ExtMetalSurfaceFn::name()
}

#[cfg(target_os = "windows")]
fn surface_extension_name() -> &'static CStr {
    Win32Surface::name()
}

#[derive(Clone)]
pub struct Vulkan {
    _debug_callback: DebugUtilsMessengerEXT,
    library: Entry,
    instance: Instance,
}

impl Vulkan {
    pub fn new(name: &str, layers: &[String], extensions: &[String]) -> Self {
        let c_name = CString::new(name).unwrap();
        let appinfo = ApplicationInfo::builder()
            .application_name(&c_name)
            .application_version(0)
            .engine_name(&c_name)
            .engine_version(0)
            .api_version(make_version(1, 2, 0));

        let layers_names_raw: Vec<*const i8> = layers
            .iter()
            .map(|layer_name| unsafe { CString::from_raw(layer_name.as_ptr() as *mut i8).as_ptr() })
            .collect();

        let surface_extensions = vec![Surface::name(), surface_extension_name()];
        let mut extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        extension_names_raw.push(DebugUtils::name().as_ptr());
        for ext in extensions.iter() {
            extension_names_raw.push(unsafe { CString::from_raw(ext.as_ptr() as *mut i8).as_ptr() })
        }

        let create_info = InstanceCreateInfo::builder()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names_raw);

        unsafe {
            let library = Entry::new().unwrap();
            let instance: Instance = library
                .create_instance(&create_info, None)
                .expect("Instance creation error");

            let debug_info = DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(DebugUtilsMessageTypeFlagsEXT::all())
                .pfn_user_callback(Some(vulkan_debug_callback));

            let debug_utils_loader = DebugUtils::new(&library, &instance);
            let debug_callback = debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap();

            Self {
                _debug_callback: debug_callback,
                library,
                instance,
            }
        }
    }

    pub fn library(&self) -> &Entry {
        &self.library
    }
    pub fn vk_instance(&self) -> &Instance {
        &self.instance
    }

    pub fn hardware_devices_with_queue_support(&self, flags: QueueFlags) -> Vec<Gpu> {
        unsafe {
            self.instance
                .enumerate_physical_devices()
                .expect("Physical device error")
                .iter()
                .map(|pdevice| {
                    self.instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, ref info)| {
                            if info.queue_flags.contains(flags) {
                                Some(Gpu::new(self, pdevice, index as u32))
                            } else {
                                None
                            }
                        })
                        .next()
                })
                .filter_map(|v| v)
                .collect::<Vec<Gpu>>()
        }
    }
}
