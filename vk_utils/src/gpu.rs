use ash::version::InstanceV1_0;
use ash::vk::{
    PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceLimits, PhysicalDeviceProperties,
    PhysicalDeviceType, QueueFamilyProperties, QueueFlags,
};

use crate::device_context::DeviceContext;
use crate::vk_instance::Vulkan;
use std::ffi::CStr;

pub struct Gpu {
    vulkan: Vulkan,
    physical_device: PhysicalDevice,
    features: PhysicalDeviceFeatures,
    properties: PhysicalDeviceProperties,
    queue_family_properties: Vec<QueueFamilyProperties>,
}

impl Gpu {
    pub(crate) fn new(vulkan: &Vulkan, physical_device: &PhysicalDevice) -> Self {
        unsafe {
            let features = vulkan
                .vk_instance()
                .get_physical_device_features(*physical_device);

            let properties = vulkan
                .vk_instance()
                .get_physical_device_properties(*physical_device);

            Self {
                vulkan: vulkan.clone(),
                features,
                properties,
                physical_device: physical_device.clone(),
                queue_family_properties: vulkan
                    .vk_instance()
                    .get_physical_device_queue_family_properties(*physical_device),
            }
        }
    }

    pub fn device_context(&self, extensions: &[&'static CStr]) -> DeviceContext {
        DeviceContext::new(&self.vulkan.vk_instance(), self, extensions)
    }

    pub(crate) fn family_type_index(&self, flags: QueueFlags) -> Option<u32> {
        for (index, queue_info) in self.queue_family_properties.iter().enumerate() {
            if queue_info.queue_flags.contains(flags) {
                return Some(index as u32);
            }
        }

        None
    }

    pub fn vk_physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    pub fn name(&self) -> String {
        let c_str = unsafe { CStr::from_ptr(self.properties.device_name.as_ptr()) };
        String::from(c_str.to_str().expect("String conversion failed"))
    }

    pub fn vendor_id(&self) -> u32 {
        self.properties.vendor_id
    }
    pub fn device_id(&self) -> u32 {
        self.properties.device_id
    }

    pub fn driver_version(&self) -> u32 {
        self.properties.driver_version
    }

    pub fn is_discrete(&self) -> bool {
        self.properties.device_type == PhysicalDeviceType::DISCRETE_GPU
    }

    pub fn is_virtual(&self) -> bool {
        self.properties.device_type == PhysicalDeviceType::VIRTUAL_GPU
    }

    pub fn limits(&self) -> PhysicalDeviceLimits {
        self.properties.limits
    }

    pub fn queue_family_count(&self) -> u32 {
        self.queue_family_properties.len() as u32
    }

    pub fn queue_count(&self, queue_family_index: u32) -> u32 {
        self.queue_family_properties[queue_family_index as usize].queue_count
    }

    pub fn supports_graphics(&self) -> bool {
        for queue_info in self.queue_family_properties.iter() {
            if queue_info.queue_flags.contains(QueueFlags::GRAPHICS) {
                return true;
            }
        }

        false
    }

    pub fn supports_compute(&self) -> bool {
        for queue_info in self.queue_family_properties.iter() {
            if queue_info.queue_flags.contains(QueueFlags::COMPUTE) {
                return true;
            }
        }

        false
    }

    pub fn supports_transfer(&self) -> bool {
        for queue_info in self.queue_family_properties.iter() {
            if queue_info.queue_flags.contains(QueueFlags::TRANSFER) {
                return true;
            }
        }

        false
    }
}
