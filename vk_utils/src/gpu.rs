use ash::version::InstanceV1_0;
use ash::vk::{
    PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceLimits, PhysicalDeviceProperties,
    PhysicalDeviceType, QueueFlags,
};
use ash::{Device, Instance};

use crate::vk_instance::Vulkan;
use std::ffi::CString;

pub struct Gpu {
    vulkan: Vulkan,
    physical_device: PhysicalDevice,
    features: PhysicalDeviceFeatures,
    properties: PhysicalDeviceProperties,
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
            }
        }
    }

    pub fn vk_physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    pub fn name(&self) -> String {
        let c_str = unsafe { CString::from_raw(self.properties.device_name.as_ptr() as *mut i8) };
        c_str.into_string().expect("String conversion failed")
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

    pub fn supports_graphics(&self) -> bool {
        unsafe {
            for queue_info in self
                .vulkan
                .vk_instance()
                .get_physical_device_queue_family_properties(self.physical_device)
                .iter()
            {
                if queue_info.queue_flags.contains(QueueFlags::GRAPHICS) {
                    return true;
                }
            }
        }

        false
    }

    pub fn supports_compute(&self) -> bool {
        unsafe {
            for queue_info in self
                .vulkan
                .vk_instance()
                .get_physical_device_queue_family_properties(self.physical_device)
                .iter()
            {
                if queue_info.queue_flags.contains(QueueFlags::COMPUTE) {
                    return true;
                }
            }
        }

        false
    }

    pub fn supports_transfer(&self) -> bool {
        unsafe {
            for queue_info in self
                .vulkan
                .vk_instance()
                .get_physical_device_queue_family_properties(self.physical_device)
                .iter()
            {
                if queue_info.queue_flags.contains(QueueFlags::TRANSFER) {
                    return true;
                }
            }
        }

        false
    }
}
