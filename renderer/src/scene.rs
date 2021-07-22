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
            maps: glm::ivec4(-1, -1, -1, -1),
        }
    }
}

pub struct Scene {
    geometry_buffer: GeometryBuffer,
    geometry_views: Vec<GeometryBufferView>,
    geometry_instances: Vec<GeometryInstance>,
    materials: Vec<Material>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            geometry_buffer: GeometryBuffer::new(),
            geometry_views: Vec::new(),
            geometry_instances: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn add_geometry(&mut self, indices: Vec<u32>, vertices: Vec<Vertex>) -> usize {
        let (index_offset, vertex_offset) = if let Some(view) = self.geometry_views.last() {
            (
                view.index_offset() + indices.len() as u32,
                view.vertex_offset() + vertices.len() as u32,
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
        instance_id as usize
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

    pub fn geometry_instances(&self) -> &[GeometryInstance] {
        &self.geometry_instances
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }
}
