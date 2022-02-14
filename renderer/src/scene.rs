use crate::geometry::*;
use ash::vk::GeometryInstanceFlagsKHR;

#[repr(C)]
pub struct Material {
    color: glm::Vec4,
    emission: glm::Vec4,
    maps: glm::IVec4,
}

impl Material {
    pub fn new() -> Self {
        Self {
            color: glm::vec4(1., 1., 1., 1.),
            emission: glm::vec4(0., 0., 0., 0.),
            maps: glm::vec4(-1, -1, -1, -1),
        }
    }
}

pub struct Scene {
    geometry_buffer: GeometryBuffer,
    geometry_views: Vec<GeometryBufferView>,
    geometry_instances: Vec<GeometryInstance>,
    geometry_instance_offsets: Vec<GeometryOffset>,
    materials: Vec<Material>,
}

unsafe impl Send for Scene {}

impl Scene {
    pub fn new() -> Self {
        Self {
            geometry_buffer: GeometryBuffer::new(),
            geometry_views: Vec::new(),
            geometry_instances: Vec::new(),
            geometry_instance_offsets: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn add_geometry(&mut self, indices: &[u32], vertices: &[Vertex]) -> usize {
        let (index_offset, vertex_offset) = if let Some(view) = self.geometry_views.last() {
            (
                view.index_offset() + view.index_count(),
                view.vertex_offset() + view.vertex_count(),
            )
        } else {
            (0, 0)
        };

        self.geometry_views.push(GeometryBufferView::new(
            indices.len() as u32,
            index_offset,
            vertices.len() as u32,
            vertex_offset,
        ));
        self.geometry_buffer.append(indices, vertices);
        return self.geometry_views.len() - 1;
    }

    pub fn create_instance(&mut self, geometry_id: usize) -> usize {
        let instance_id = self.geometry_instances.len() as u32;
        self.geometry_instances.push(GeometryInstance::new(
            instance_id,
            0xff,
            0,
            GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE,
            geometry_id as u64,
        ));

        self.materials.push(Material::new());
        self.geometry_instance_offsets.push(GeometryOffset {
            index: self.geometry_views[geometry_id].index_offset(),
            vertex: self.geometry_views[geometry_id].vertex_offset(),
        });
        self.set_scale(instance_id as usize, &glm::Vec3::new(1.0, 1.0, 1.0));
        instance_id as usize
    }

    pub fn set_transform(&mut self, instance_id: usize, transform: &glm::Mat4x3) {
        self.geometry_instances[instance_id].transform = *transform;
    }

    pub fn set_orientation(&mut self, instance_id: usize, orientation: &glm::Quat) {
        let rotation_matrix = glm::quat_to_mat4(orientation);
        //self.geometry_instances[instance_id].transform *= &rotation_matrix;
    }

    pub fn set_position(&mut self, instance_id: usize, position: &glm::Vec3) {
        self.geometry_instances[instance_id].transform[3] = position.x;
        self.geometry_instances[instance_id].transform[7] = position.y;
        self.geometry_instances[instance_id].transform[11] = position.z;
    }

    pub fn set_scale(&mut self, instance_id: usize, scale: &glm::Vec3) {
        self.geometry_instances[instance_id].transform[0] = scale.x;
        self.geometry_instances[instance_id].transform[5] = scale.y;
        self.geometry_instances[instance_id].transform[10] = scale.z;
    }

    pub fn geometry_buffer(&self) -> &GeometryBuffer {
        &self.geometry_buffer
    }
    pub fn geometry_count(&self) -> usize {
        self.geometry_views.len()
    }

    pub fn geometry_buffer_views(&self) -> &[GeometryBufferView] {
        &self.geometry_views
    }

    pub fn geometry_offsets(&self) -> &[GeometryOffset] {
        &self.geometry_instance_offsets
    }

    pub fn geometry_instances(&self) -> &[GeometryInstance] {
        &self.geometry_instances
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }
}
