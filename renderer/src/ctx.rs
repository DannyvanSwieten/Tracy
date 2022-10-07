use std::{collections::HashMap, rc::Rc};

use ash::vk::{GeometryInstanceFlagsKHR, QueueFlags};
use vk_utils::{device_context::DeviceContext, queue::CommandQueue};

use crate::{
    context::RtxExtensions,
    geometry::{GeometryInstance, Normal, Position, Tangent, Texcoord},
    material::Material,
    mesh::{Mesh, MeshAddress},
};

struct InstanceProperties {
    material_id: u32,
    geometry_id: u32,
}

pub struct RenderContext {
    device: Rc<DeviceContext>,
    rtx: RtxExtensions,
    queue: Rc<CommandQueue>,
    meshes: Vec<Mesh>,
    mesh_addresses: Vec<MeshAddress>,
    instances: HashMap<usize, Vec<GeometryInstance>>,
    instance_properties: HashMap<usize, Vec<InstanceProperties>>,
    materials: Vec<Material>,
}

impl RenderContext {
    pub fn new_with_shared_device(device: Rc<DeviceContext>) -> Self {
        Self {
            device: device.clone(),
            rtx: RtxExtensions::new(&device),
            queue: Rc::new(CommandQueue::new(device, QueueFlags::GRAPHICS)),
            meshes: Vec::new(),
            mesh_addresses: Vec::new(),
            instances: HashMap::new(),
            instance_properties: HashMap::new(),
            materials: Vec::new(),
        }
    }

    pub fn create_mesh(
        &mut self,
        indices: &[u32],
        positions: &[Position],
        normals: &[Normal],
        tangents: &[Tangent],
        tex_coords: &[Texcoord],
    ) -> usize {
        self.meshes.push(Mesh::new(
            self.device.clone(),
            &self.rtx,
            self.queue.clone(),
            indices,
            positions,
            normals,
            tangents,
            tex_coords,
        ));

        self.mesh_addresses
            .push(MeshAddress::new(self.meshes.last().unwrap()));

        self.meshes.len() - 1
    }

    fn create_instance(&mut self, geometry_id: usize) -> InstanceHandle {
        let blas = self.meshes[geometry_id].blas.address();
        let instance_id = self.instances[&geometry_id].len();
        self.instances
            .get_mut(&geometry_id)
            .unwrap()
            .push(GeometryInstance::new(
                instance_id as u32,
                0xff,
                0,
                GeometryInstanceFlagsKHR::FORCE_OPAQUE,
                blas,
                glm::Mat4x3::default(),
            ));
        InstanceHandle {
            ctx: self,
            geometry_id,
            instance_id,
        }
    }

    fn set_instance_material(
        &mut self,
        geometry_id: usize,
        instance_id: usize,
        material_id: usize,
    ) {
        self.instance_properties.get_mut(&geometry_id).unwrap()[instance_id].material_id =
            material_id as u32;
    }
}

pub struct InstanceHandle<'a> {
    ctx: &'a mut RenderContext,
    geometry_id: usize,
    instance_id: usize,
}

impl<'a> InstanceHandle<'a> {
    pub fn set_material(&mut self, material_id: usize) {
        self.ctx
            .set_instance_material(self.geometry_id, self.instance_id, material_id);
    }
}
