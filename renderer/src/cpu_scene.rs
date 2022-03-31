use std::sync::Arc;

use crate::{
    geometry::*,
    gpu_scene::GpuTexture,
    resource::{GpuResource, Resource},
};
use glm::{vec2, vec3, vec4};
#[derive(Clone, Copy)]
pub struct Camera {
    pub fov: f32,
    pub z_near: f32,
    pub z_far: f32,
}
#[repr(C)]
#[derive(Clone)]
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
                new_pixels.push(pixels[i]);
                new_pixels.push(pixels[i + 1]);
                new_pixels.push(pixels[i + 2]);
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

impl GpuResource for TextureImageData {
    type Item = GpuTexture;

    fn prepare(
        &self,
        device: &vk_utils::device_context::DeviceContext,
        rtx: &crate::context::RtxContext,
    ) -> Self::Item {
        todo!()
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Material {
    pub base_color: glm::Vec4,
    pub emission: glm::Vec4,
    pub roughness: f32,
    pub metalness: f32,
    pub albedo_map: Option<Arc<Resource<TextureImageData>>>,
    pub normal_map: Option<Arc<Resource<TextureImageData>>>,
    pub metallic_roughness_map: Option<Arc<Resource<TextureImageData>>>,
    pub emission_map: Option<Arc<Resource<TextureImageData>>>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: vec4(0.5, 0.5, 0.5, 1.),
            roughness: 1.0,
            metalness: 0.0,
            emission: vec4(0., 0., 0., 0.),
            albedo_map: None,
            normal_map: None,
            metallic_roughness_map: None,
            emission_map: None,
        }
    }
}

impl Material {
    pub fn new(color: &glm::Vec4) -> Self {
        Self {
            base_color: *color,
            ..Default::default()
        }
    }
}

impl GpuResource for Material {
    type Item = u32;

    fn prepare(
        &self,
        device: &vk_utils::device_context::DeviceContext,
        rtx: &crate::context::RtxContext,
    ) -> Self::Item {
        todo!()
    }
}

#[derive(Default, Clone)]
pub struct SceneGraphNode {
    pub name: String,
    pub camera: Option<usize>,
    pub mesh: Option<Vec<usize>>,
    pub children: Vec<usize>,
    pub global_transform: [[f32; 4]; 4],
    pub local_transform: [[f32; 4]; 4],
}

impl SceneGraphNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn with_camera(mut self, camera_id: usize) -> Self {
        self.camera = Some(camera_id);
        self
    }

    pub fn with_mesh(mut self, mesh_id: usize) -> Self {
        if let Some(vector) = self.mesh.as_mut() {
            vector.push(mesh_id)
        } else {
            self.mesh = Some(vec![mesh_id])
        }
        self
    }

    pub fn with_children(mut self, children: &[usize]) -> Self {
        self.children.extend(children);
        self
    }
}

pub struct Scene {
    pub name: String,
    geometry_buffer: GeometryBuffer,
    geometry_views: Vec<GeometryBufferView>,
    geometry_instances: Vec<GeometryInstance>,
    geometry_instance_offsets: Vec<GeometryOffset>,

    pub images: Vec<TextureImageData>,
    pub cameras: Vec<Camera>,
    pub materials: Vec<Material>,
    pub material_names: Vec<String>,

    pub root: usize,
    pub nodes: Vec<SceneGraphNode>,
}

impl Default for Scene {
    fn default() -> Self {
        let mut node = SceneGraphNode::default();
        node.name = "Root".to_string();
        let mut scene = Self {
            name: "Default".to_string(),
            geometry_buffer: Default::default(),
            geometry_views: Default::default(),
            geometry_instances: Default::default(),
            geometry_instance_offsets: Default::default(),
            materials: Default::default(),
            images: Default::default(),
            cameras: Default::default(),
            material_names: Default::default(),
            nodes: vec![node],
            root: 0,
        };

        scene.create_floor(-5.);
        scene
    }
}

impl Scene {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            geometry_buffer: GeometryBuffer::new(),
            geometry_views: Vec::new(),
            geometry_instances: Vec::new(),
            geometry_instance_offsets: Vec::new(),
            materials: Vec::new(),
            images: Vec::new(),
            cameras: Vec::new(),
            nodes: Vec::new(),
            material_names: Vec::new(),
            root: 0,
        }
    }

    pub fn add_node(&mut self, node: SceneGraphNode) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn add_child_to_node(&mut self, parent_id: usize, child_id: usize) {
        self.nodes[parent_id].children.push(child_id)
    }

    pub fn add_camera(&mut self, camera: &Camera) -> usize {
        self.cameras.push(*camera);
        self.cameras.len() - 1
    }

    pub fn add_material(&mut self, name: &str, material: &Material) -> usize {
        self.materials.push(material.clone());
        self.material_names.push(name.to_string());
        self.materials.len() - 1
    }

    pub fn node(&mut self, id: usize) -> &mut SceneGraphNode {
        &mut self.nodes[id]
    }

    pub fn add_image(
        &mut self,
        format: ash::vk::Format,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> usize {
        self.images
            .push(TextureImageData::new(format, width, height, data));
        self.images.len() - 1
    }

    pub fn create_floor(&mut self, y: f32) {
        let floor_id = self.add_geometry(
            "Floor",
            &[0, 2, 1, 0, 3, 2],
            &[
                Vertex::new(-1.0, y, 1.0),
                Vertex::new(-1.0, y, -1.0),
                Vertex::new(1.0, y, -1.0),
                Vertex::new(1.0, y, 1.0),
            ],
            &[
                vec3(0., 1., 0.),
                vec3(0., 1., 0.),
                vec3(0., 1., 0.),
                vec3(0., 1., 0.),
            ],
            &[
                vec3(1., 0., 0.),
                vec3(1., 0., 0.),
                vec3(1., 0., 0.),
                vec3(1., 0., 0.),
            ],
            &[
                vec2(0.0, 1.0),
                vec2(0.0, 0.0),
                vec2(1.0, 0.0),
                vec2(1.0, 1.0),
            ],
        );
        let instance_id = 0;
        self.set_scale(instance_id, &vec3(1000.0, 1.0, 1000.0));

        let node_id = self.add_node(SceneGraphNode::new("Floor").with_mesh(floor_id));
        self.add_child_to_node(self.root, node_id);
    }

    pub fn add_geometry(
        &mut self,
        name: &str,
        indices: &[u32],
        vertices: &[Vertex],
        normals: &[nalgebra_glm::Vec3],
        tangents: &[nalgebra_glm::Vec3],
        tex_coords: &[nalgebra_glm::Vec2],
    ) -> usize {
        let (index_offset, vertex_offset) = if let Some(view) = self.geometry_views.last() {
            (
                view.index_offset() + view.index_count(),
                view.vertex_offset() + view.vertex_count(),
            )
        } else {
            (0, 0)
        };

        self.geometry_views.push(GeometryBufferView::new(
            name,
            indices.len() as u32,
            index_offset,
            vertices.len() as u32,
            vertex_offset,
        ));
        self.geometry_buffer
            .append(indices, vertices, normals, tangents, tex_coords);
        return self.geometry_views.len() - 1;
    }

    pub fn set_material_base_color(&mut self, instance_id: usize, color: &glm::Vec4) {
        //self.materials[instance_id].color = *color
    }

    pub fn set_material_base_color_texture(&mut self, instance_id: usize, texture_id: usize) {
        //self.materials[instance_id].maps[0] = texture_id as i32;
    }

    pub fn set_material_metallic(&mut self, instance_id: usize, metallic: f32) {
        //self.materials[instance_id].metallic_roughness[1] = metallic;
    }

    pub fn set_material_roughness(&mut self, instance_id: usize, roughness: f32) {
        //self.materials[instance_id].metallic_roughness[0] = roughness;
    }

    pub fn set_material_metallic_roughness_texture(
        &mut self,
        instance_id: usize,
        texture_id: usize,
    ) {
        //self.materials[instance_id].maps[1] = texture_id as i32;
    }

    pub fn set_material_emission(&mut self, instance_id: usize, color: &glm::Vec3, intensity: f32) {
        self.materials[instance_id].emission = vec4(color[0], color[1], color[2], intensity);
    }

    pub fn set_material_emission_texture(&mut self, instance_id: usize, texture_id: usize) {
        //self.materials[instance_id].maps[3] = texture_id as i32;
    }

    pub fn set_material(&mut self, instance_id: usize, material: &Material) {
        self.materials[instance_id] = material.clone();
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

    pub fn set_scale(&mut self, instance_id: usize, scale: &glm::Vec3) {
        self.geometry_instances[instance_id].transform[0] = scale.x;
        self.geometry_instances[instance_id].transform[5] = scale.y;
        self.geometry_instances[instance_id].transform[10] = scale.z;
    }

    pub fn set_scale_values(&mut self, instance_id: usize, x: f32, y: f32, z: f32) {
        self.geometry_instances[instance_id].transform[0] = x;
        self.geometry_instances[instance_id].transform[5] = y;
        self.geometry_instances[instance_id].transform[10] = z;
    }

    pub fn set_uniform_scale(&mut self, instance_id: usize, s: f32) {
        self.geometry_instances[instance_id].transform[0] = s;
        self.geometry_instances[instance_id].transform[5] = s;
        self.geometry_instances[instance_id].transform[10] = s;
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
