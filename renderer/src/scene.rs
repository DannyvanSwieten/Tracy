use crate::geometry::*;
use ash::vk::GeometryInstanceFlagsKHR;
#[derive(Clone, Copy)]
pub struct Camera {
    pub fov: f32,
    pub z_near: f32,
    pub z_far: f32,
}
pub struct TextureImageData {
    pub format: ash::vk::Format,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl TextureImageData {
    pub fn new(format: ash::vk::Format, width: u32, height: u32, pixels: &[u8]) -> Self {
        if format == ash::vk::Format::R8G8B8_UNORM {
            let mut new_pixels = Vec::new();
            for i in (0..pixels.len()).step_by(3) {
                new_pixels.extend(&pixels[i..i + 3]);
                new_pixels.push(255);
            }
            Self {
                format: ash::vk::Format::R8G8B8A8_UNORM,
                width,
                height,
                pixels: new_pixels,
            }
        } else {
            Self {
                format,
                width,
                height,
                pixels: pixels.to_vec(),
            }
        }
    }
}

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

    images: Vec<TextureImageData>,
    cameras: Vec<Camera>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            geometry_buffer: GeometryBuffer::new(),
            geometry_views: Vec::new(),
            geometry_instances: Vec::new(),
            geometry_instance_offsets: Vec::new(),
            materials: Vec::new(),
            images: Vec::new(),
            cameras: Vec::new(),
        }
    }

    pub fn add_camera(&mut self, camera: &Camera) {
        self.cameras.push(*camera)
    }

    pub fn add_image(&mut self, format: ash::vk::Format, width: u32, height: u32, data: &[u8]) {
        self.images
            .push(TextureImageData::new(format, width, height, data))
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

    pub fn set_matrix(&mut self, instance_id: usize, matrix: &[[f32; 4]; 4]) {
        self.geometry_instances[instance_id].transform[0] = matrix[0][0];
        self.geometry_instances[instance_id].transform[1] = matrix[0][1];
        self.geometry_instances[instance_id].transform[2] = matrix[0][2];
        self.geometry_instances[instance_id].transform[3] = matrix[0][3];

        self.geometry_instances[instance_id].transform[4] = matrix[1][0];
        self.geometry_instances[instance_id].transform[5] = matrix[1][1];
        self.geometry_instances[instance_id].transform[6] = matrix[1][2];
        self.geometry_instances[instance_id].transform[7] = matrix[1][3];

        self.geometry_instances[instance_id].transform[8] = matrix[2][0];
        self.geometry_instances[instance_id].transform[9] = matrix[2][1];
        self.geometry_instances[instance_id].transform[10] = matrix[2][2];
        self.geometry_instances[instance_id].transform[11] = matrix[2][3];
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

    pub fn set_position_values(&mut self, instance_id: usize, x: f32, y: f32, z: f32) {
        self.geometry_instances[instance_id].transform[3] = x;
        self.geometry_instances[instance_id].transform[7] = y;
        self.geometry_instances[instance_id].transform[11] = z;
    }

    pub fn set_scale_values(&mut self, instance_id: usize, x: f32, y: f32, z: f32) {
        self.geometry_instances[instance_id].transform[0] = x;
        self.geometry_instances[instance_id].transform[5] = y;
        self.geometry_instances[instance_id].transform[10] = z;
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

    pub fn images(&self) -> &[TextureImageData] {
        &self.images
    }

    pub fn cameras(&self) -> &[Camera] {
        &self.cameras
    }
}
