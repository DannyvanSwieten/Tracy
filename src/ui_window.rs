// use super::application::*;
// use ash::version::InstanceV1_0;
// use ash::version::EntryV1_0;
// use ash::vk::Handle;
// use std::ptr;
// use std::ffi::c_void;
// use skia_safe::{Budgeted, Canvas, ImageInfo, Surface};
// use skia_safe::gpu::*;

// unsafe fn get_procedure(entry: &ash::Entry, instance: &ash::Instance, of: vk::GetProcOf) -> Option<unsafe extern "system" fn()> {
//     match of {
//         vk::GetProcOf::Instance(instance, name) => {
//             let ash_instance = Handle::from_raw(instance as _);
//             entry.get_instance_proc_addr(ash_instance, name)
//         },

//         vk::GetProcOf::Device(device, name) => {
//             let ash_device = Handle::from_raw(device as _);
//             instance.get_device_proc_addr(ash_device, name)
//         },
//     }
// }

// pub struct UIWindow<'a>{
//     backend: vk::BackendContext<'a>,
//     context: RecordingContext,
    

// }

// impl<'a> UIWindow<'a> {
//     fn new(app: &'static Application, width: u32, height: u32) -> Self {
//         let (queue, index) = app.present_queue_and_index();

//         let entry = app.vulkan_entry();
//         let instance = app.vulkan_instance();
//         let get_proc = move |of| unsafe {
//             if let Some(f) = get_procedure(&entry, &instance, of){
//                 f as *const std::ffi::c_void
//             } else {
//                 std::ptr::null()
//             }
//         };

//         let backend = unsafe {
//             vk::BackendContext::new(
//                 app.vulkan_instance().handle().as_raw() as _,
//                 app.primary_gpu().as_raw() as _, 
//                 app.primary_device_context().handle().as_raw() as _, 
//                 (
//                     queue.as_raw() as _,
//                     index,
//                 ),
//                 &get_proc as _, 
//             )
//         };

//         let mut context = RecordingContext::from(DirectContext::new_vulkan(&backend, None).unwrap());
//         let image_info = ImageInfo::new_n32_premul((500 * 2, 500 * 2), None);
//         let mut surface = Surface::new_render_target(
//             &mut context,
//             Budgeted::Yes,
//             &image_info,
//             None,
//             SurfaceOrigin::TopLeft,
//             None,
//             false,
//         )
//         .unwrap();

//         Self{backend, context}
//     }
// }