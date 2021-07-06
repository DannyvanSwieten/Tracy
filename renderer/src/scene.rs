use crate::geometry::*;
use ash::vk::GeometryInstanceFlagsKHR;

pub struct Scene {
    geometry_buffer: GeometryBuffer,
    geometry_views: Vec<GeometryBufferView>,
    geometry_instances: Vec<GeometryInstance>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            geometry_buffer: GeometryBuffer::new(),
            geometry_views: Vec::new(),
            geometry_instances: Vec::new(),
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

    pub fn create_instance(&mut self, id: usize) -> usize {
        let instance_id = self.geometry_instances.len() as u32;
        self.geometry_instances.push(GeometryInstance::new(
            instance_id,
            0xff,
            0,
            GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE,
            id as u64,
        ));

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
}
