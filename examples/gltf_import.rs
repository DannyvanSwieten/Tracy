use std::{collections::HashMap, path::Path, rc::Rc};

use cgmath::{vec2, vec3, vec4};
use gltf::{
    accessor::{DataType, Dimensions, Iter},
    Semantic,
};

use renderer::{
    image_resource::TextureImageData,
    math::Vec4,
    mesh_resource::MeshResource,
    vk::{self, Format, Image},
};

struct Resource<T> {
    id: String,
    data: T,
}

impl<T> Resource<T> {
    fn new(id: &str, data: T) -> Self {
        Self {
            id: id.to_owned(),
            data,
        }
    }
}

trait ResourceLoader<T> {
    fn can_load(&self, extension: &str) -> bool;
    fn load(&self, path: &Path) -> Resource<T>;
}

struct ImageLoader;
impl ResourceLoader<TextureImageData> for Image {
    fn can_load(&self, extension: &str) -> bool {
        if extension == "png" {
            return true;
        }

        false
    }

    fn load(&self, path: &Path) -> Resource<TextureImageData> {
        let data = image::open(path).unwrap();
        let texture_data = TextureImageData::new(
            Format::R8G8B8A8_UINT,
            data.width(),
            data.height(),
            data.as_bytes(),
        );
        Resource::new(path.to_str().unwrap(), texture_data)
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
    pub base_color_texture: Option<Rc<TextureImageData>>,
    pub metallic_roughness_texture: Option<Rc<TextureImageData>>,
    pub normal_texture: Option<Rc<TextureImageData>>,
    pub emission_texture: Option<Rc<TextureImageData>>,
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

fn main() {
    let duck_path = std::env::current_dir()
        .expect("No working directory found")
        .join("assets/Duck/glTF/Duck.gltf");
    let (gltf, buffers, images) = gltf::import(duck_path).expect("GLTF import failed");
    let get_buffer_data = |buffer: gltf::Buffer| buffers.get(buffer.index()).map(|x| &*x.0);
    let mut mesh_resources = HashMap::new();
    let mut image_resources = Vec::new();
    let mut material_resources = Vec::new();

    for image in images {
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

        image_resources.push(Rc::new(TextureImageData::new(
            format,
            image.width,
            image.height,
            &image.pixels,
        )));
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

        material_resources.push(Rc::new(mat))
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
            mesh_resources.insert(mesh.index(), resource);
        }

        println!("{}", mesh_resources.len())
    }
}
