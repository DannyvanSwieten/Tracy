use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    path::{Path, PathBuf},
    rc::Rc,
};

use ash::extensions::ext::DebugUtils;
use cgmath::{vec2, vec3, vec4};
use gltf::{
    accessor::{DataType, Dimensions, Iter},
    Semantic,
};

use renderer::{
    camera::Camera,
    ctx::{Ctx, Handle},
    image_resource::TextureImageData,
    math::{Quat, Vec3, Vec4},
    mesh_resource::MeshResource,
    scene::Scene,
    vk::{self, Format},
};
use vk_utils::vulkan::Vulkan;

#[derive(Clone, Copy)]
pub struct Transform {
    orientation: Quat,
    translation: Vec3,
    scale: Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            orientation: Quat::new(1.0, 0.0, 0.0, 0.0),
            translation: vec3(0.0, 0.0, 0.0),
            scale: vec3(1.0, 1.0, 1.0),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl From<([f32; 3], [f32; 4], [f32; 3])> for Transform {
    fn from((t, quat, s): ([f32; 3], [f32; 4], [f32; 3])) -> Self {
        Self {
            orientation: Quat::new(quat[0], quat[1], quat[2], quat[3]),
            translation: vec3(t[0], t[1], t[2]),
            scale: vec3(s[0], s[1], s[2]),
        }
    }
}

pub struct Actor {
    pub transform: Transform,
    pub mesh: Option<Rc<ResourceHandle<MeshResource>>>,
    pub material: Option<Rc<ResourceHandle<ImportedMaterial>>>,
    pub children: Vec<usize>,
}

impl Actor {
    pub fn new() -> Self {
        Self {
            transform: Transform::default(),
            mesh: None,
            material: None,
            children: Vec::new(),
        }
    }

    pub fn with_transform(mut self, t: Transform) -> Self {
        self.transform = t;
        self
    }

    pub fn with_mesh(mut self, mesh: Rc<ResourceHandle<MeshResource>>) -> Self {
        self.mesh = Some(mesh);
        self
    }

    pub fn with_material(mut self, material: Rc<ResourceHandle<ImportedMaterial>>) -> Self {
        self.material = Some(material);
        self
    }
}

impl Default for Actor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SceneGraph {
    actors: Vec<Actor>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self { actors: Vec::new() }
    }

    pub fn add_actor(&mut self, actor: Actor) -> usize {
        self.actors.push(actor);
        self.actors.len() - 1
    }

    pub fn actors(&self) -> &Vec<Actor> {
        &self.actors
    }

    pub fn get_mut(&mut self, id: usize) -> &mut Actor {
        &mut self.actors[id]
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}
struct ResourceManager {
    resources: HashMap<std::any::TypeId, HashMap<String, Rc<dyn Any>>>,
}

#[derive(Clone)]
pub struct ResourceHandle<T> {
    path: String,
    _ph: PhantomData<T>,
}

impl<T> ResourceHandle<T> {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            _ph: PhantomData::default(),
        }
    }
}

pub struct ResourceCollection<'a, T> {
    data: &'a HashMap<String, Rc<dyn Any>>,
    _ph: std::marker::PhantomData<T>,
}

impl<'a, T: 'static> ResourceCollection<'a, T> {
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&str, &T),
    {
        for (id, any) in self.data {
            let resource = any.downcast_ref::<T>();
            if let Some(resource) = resource {
                f(id, resource)
            }
        }
    }
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::default(),
        }
    }

    pub fn register<T: Sized + 'static>(
        &mut self,
        path: &str,
        resource: T,
    ) -> Rc<ResourceHandle<T>> {
        let any: Rc<dyn Any> = Rc::new(resource);
        self.resources
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(HashMap::new)
            .insert(path.to_owned(), any);
        Rc::new(ResourceHandle {
            path: path.to_owned(),
            _ph: std::marker::PhantomData::default(),
        })
    }

    pub fn get<T: 'static>(&self, handle: &ResourceHandle<T>) -> Option<&T> {
        let id = TypeId::of::<T>();
        if let Some(map) = self.resources.get(&id) {
            if let Some(any) = map.get(&handle.path) {
                any.downcast_ref::<T>()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_all<T: 'static>(&self) -> Option<ResourceCollection<T>> {
        let id = TypeId::of::<T>();
        if let Some(map) = self.resources.get(&id) {
            Some(ResourceCollection {
                data: map,
                _ph: PhantomData::default(),
            })
        } else {
            None
        }
    }
}

struct ResourceCache {
    resources: HashMap<String, Handle>,
}

impl ResourceCache {
    pub fn new() -> Self {
        Self {
            resources: HashMap::default(),
        }
    }

    pub fn register(&mut self, path: &str, gpu_handle: Handle) {
        self.resources.insert(path.to_owned(), gpu_handle);
    }
}

trait ResourceLoader<T> {
    fn can_load(&self, extension: &str) -> bool;
    fn load(&self, path: &Path) -> T;
}

struct ImageLoader;
impl ResourceLoader<TextureImageData> for ImageLoader {
    fn can_load(&self, extension: &str) -> bool {
        if extension.to_lowercase() == "png"
            || extension.to_lowercase() == "jpg"
            || extension.to_lowercase() == "jpeg"
            || extension.to_lowercase() == "exr"
        {
            return true;
        }

        false
    }

    fn load(&self, path: &Path) -> TextureImageData {
        let data = image::open(path).unwrap();
        let texture_data = TextureImageData::new(
            Format::R8G8B8A8_UINT,
            data.width(),
            data.height(),
            data.as_bytes(),
        );
        texture_data
    }
}

pub struct ImportedMaterial {
    pub base_color: Vec4,
    pub emission: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub sheen: f32,
    pub clear_coat: f32,
    pub ior: f32,
    pub transmission: f32,
    pub base_color_texture: Option<Rc<ResourceHandle<TextureImageData>>>,
    pub metallic_roughness_texture: Option<Rc<ResourceHandle<TextureImageData>>>,
    pub normal_texture: Option<Rc<ResourceHandle<TextureImageData>>>,
    pub emission_texture: Option<Rc<ResourceHandle<TextureImageData>>>,
}

impl Default for ImportedMaterial {
    fn default() -> Self {
        Self {
            base_color: vec4(0.75, 0.75, 0.75, 0.75),
            emission: vec4(0.0, 0.0, 0.0, 0.0),
            roughness: 0.5,
            metallic: 0.0,
            sheen: 0.0,
            clear_coat: 0.0,
            ior: 1.0,
            transmission: 0.0,
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_texture: None,
            emission_texture: None,
        }
    }
}

fn load_gltf(gltf_path: &PathBuf, loader: &mut ResourceManager) -> SceneGraph {
    let (gltf, buffers, images) = gltf::import(&gltf_path).expect("GLTF import failed");
    let get_buffer_data = |buffer: gltf::Buffer| buffers.get(buffer.index()).map(|x| &*x.0);
    let mut mesh_resources = HashMap::new();
    let mut image_resources = Vec::new();
    let mut material_resources = Vec::new();
    let mut node_to_actors = HashMap::new();

    for (index, image) in images.iter().enumerate() {
        let format = match image.format {
            gltf::image::Format::R8 => vk::Format::R8_UINT,
            gltf::image::Format::R8G8 => vk::Format::R8G8_UINT,
            gltf::image::Format::R8G8B8 => vk::Format::R8G8B8_UINT,
            gltf::image::Format::R8G8B8A8 => vk::Format::R8G8B8A8_UINT,
            gltf::image::Format::B8G8R8 => todo!(),
            gltf::image::Format::B8G8R8A8 => vk::Format::B8G8R8A8_UINT,
            gltf::image::Format::R16 => vk::Format::R16_SFLOAT,
            gltf::image::Format::R16G16 => vk::Format::R16G16_SFLOAT,
            gltf::image::Format::R16G16B16 => todo!(),
            gltf::image::Format::R16G16B16A16 => vk::Format::R16G16B16A16_SFLOAT,
        };

        let resource_path =
            gltf_path.to_str().unwrap().to_owned() + "#textures/" + &index.to_string();
        image_resources.push(loader.register(
            &resource_path,
            TextureImageData::new(format, image.width, image.height, &image.pixels),
        ));
    }

    for material in gltf.materials() {
        let mut mat = ImportedMaterial::default();
        let pbr = material.pbr_metallic_roughness();
        mat.base_color = Vec4::from(pbr.base_color_factor());
        if let Some(texture) = pbr.base_color_texture() {
            let resource = image_resources[texture.texture().index()].clone();
            mat.base_color_texture = Some(resource);
        }

        if let Some(texture) = material.normal_texture() {
            let resource = image_resources[texture.texture().index()].clone();
            mat.normal_texture = Some(resource);
        }

        mat.roughness = pbr.roughness_factor();
        mat.metallic = pbr.metallic_factor();
        if let Some(texture) = pbr.metallic_roughness_texture() {
            let resource = image_resources[texture.texture().index()].clone();
            mat.metallic_roughness_texture = Some(resource);
        }

        let emissive_factor = material.emissive_factor();
        mat.emission = vec4(
            emissive_factor[0],
            emissive_factor[1],
            emissive_factor[2],
            1.0,
        );
        if let Some(texture) = material.emissive_texture() {
            let resource = image_resources[texture.texture().index()].clone();
            mat.emission_texture = Some(resource);
        }

        let handle = if let Some(index) = material.index() {
            let resource_path =
                gltf_path.to_str().unwrap().to_owned() + "#materials/" + &index.to_string();
            loader.register(&resource_path, mat)
        } else {
            Rc::new(ResourceHandle::<ImportedMaterial>::new("materials/default"))
        };

        material_resources.push(handle)
    }

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let mut vertices = Vec::new();
            let mut normals = Vec::new();
            let mut tangents = Vec::new();
            let mut texcoords = Vec::new();
            let mut indices = Vec::new();
            for (semantic, accessor) in primitive.attributes() {
                match (semantic, accessor.data_type(), accessor.dimensions()) {
                    (Semantic::Positions, DataType::F32, Dimensions::Vec3) => {
                        let iter = Iter::<[f32; 3]>::new(accessor, get_buffer_data);
                        iter.into_iter().for_each(|item| {
                            for f in item {
                                vertices.push(vec3(f[0], f[1], f[2]))
                            }
                        });
                    }
                    (Semantic::Normals, DataType::F32, Dimensions::Vec3) => {
                        let iter = Iter::<[f32; 3]>::new(accessor, get_buffer_data);
                        iter.into_iter().for_each(|item| {
                            for f in item {
                                normals.push(vec3(f[0], f[1], f[2]))
                            }
                        });
                    }
                    (Semantic::Tangents, DataType::F32, Dimensions::Vec3) => {
                        let iter = Iter::<[f32; 3]>::new(accessor, get_buffer_data);
                        iter.into_iter().for_each(|item| {
                            for f in item {
                                tangents.push(vec3(f[0], f[1], f[2]))
                            }
                        });
                    }
                    (Semantic::TexCoords(0), DataType::F32, Dimensions::Vec2) => {
                        let iter = Iter::<[f32; 2]>::new(accessor, get_buffer_data);
                        iter.into_iter().for_each(|item| {
                            for f in item {
                                texcoords.push(vec2(f[0], f[1]))
                            }
                        });
                    }
                    _ => {}
                }
            }

            if let Some(accessor) = primitive.indices() {
                if accessor.data_type() == DataType::U32 {
                    let iter = Iter::<u32>::new(accessor, get_buffer_data);
                    iter.into_iter().for_each(|item| {
                        for index in item {
                            indices.push(index)
                        }
                    })
                } else {
                    let iter = Iter::<u16>::new(accessor, get_buffer_data);
                    iter.into_iter().for_each(|item| {
                        for index in item {
                            indices.push(index as u32)
                        }
                    })
                }
            } else {
                for i in 0..vertices.len() {
                    indices.push(i as u32)
                }
            }

            if tangents.is_empty() {
                for _ in 0..vertices.len() {
                    tangents.push(vec3(0.0, 0.0, 0.0))
                }
            }

            let resource = MeshResource::new(indices, vertices, normals, tangents, texcoords);
            let resource_path = gltf_path.to_str().unwrap().to_owned()
                + "#meshes/"
                + &mesh.index().to_string()
                + "/primitive/"
                + &primitive.index().to_string();
            mesh_resources.insert(
                (mesh.index(), primitive.index()),
                loader.register(&resource_path, resource),
            );
        }
    }

    let mut scene_graph = SceneGraph::new();
    for node in gltf.nodes() {
        let mut actors = Vec::new();
        let transform = Transform::from(node.transform().decomposed());
        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                let mesh_resource = mesh_resources
                    .get(&(mesh.index(), primitive.index()))
                    .unwrap();
                let material_resource = &material_resources[primitive.material().index().unwrap()];
                let actor = Actor::new()
                    .with_transform(transform)
                    .with_mesh(mesh_resource.clone())
                    .with_material(material_resource.clone());

                actors.push(scene_graph.add_actor(actor));
            }
        } else {
            let actor = Actor::new().with_transform(transform);
            actors.push(scene_graph.add_actor(actor));
        }

        node_to_actors.insert(node.index(), actors);
    }

    for node in gltf.nodes() {
        let parent_primitives = node_to_actors.get(&node.index()).unwrap();
        let parent_id = parent_primitives[0];
        let parent_node = scene_graph.get_mut(parent_id);
        if parent_primitives.len() > 1 {
            for primitive_id in 1..parent_primitives.len() {
                parent_node.children.push(primitive_id)
            }
        }
        for child in node.children() {
            let children = node_to_actors.get(&child.index()).unwrap();
            for child in children {
                parent_node.children.push(*child);
            }
        }
    }

    scene_graph
}

fn main() {
    let vulkan = Vulkan::new(
        "tracey renderer",
        &[std::ffi::CString::new("VK_LAYER_KHRONOS_validation").expect("String Creation Failed")],
        &[DebugUtils::name()],
    );
    let gpu = &vulkan.hardware_devices_with_queue_support(ash::vk::QueueFlags::GRAPHICS)[0];
    let device = if cfg!(unix) {
        Rc::new(Ctx::create_suitable_device_mac(gpu))
    } else {
        Rc::new(Ctx::create_suitable_device_windows(gpu))
    };

    let image_width = 1280;
    let image_height = 720;
    let mut ctx = Ctx::new(device, 1);
    let mut framebuffer = ctx.create_framebuffer(image_width, image_height);
    let mut scene = Scene::new();
    let mut camera = Camera::new(45.0, 0.01, 1000.0);
    camera.translate(vec3(0.0, 0.0, -10.0));
    scene.set_camera(camera);

    let gltf_path = std::env::current_dir()
        .expect("No working directory found")
        .join("assets/Sponza/glTF/Sponza.gltf");

    let mut loader = ResourceManager::new();
    let mut cache = ResourceCache::new();
    let scene_graph = load_gltf(&gltf_path, &mut loader);
    if let Some(textures) = loader.get_all::<TextureImageData>() {
        textures.for_each(|path, data| {
            let handle = ctx.create_texture(data);
            cache.register(path, handle)
        })
    }

    println!("{}", 100);
}
