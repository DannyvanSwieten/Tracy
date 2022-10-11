use std::collections::HashMap;
use std::sync::Arc;

use nalgebra_glm::{vec2, vec3, vec4, Mat4};
use renderer::{
    cpu_resource_cache::Resources, geometry::Position, gpu_resource::CpuResource,
    image_resource::TextureImageData, material_resource::Material, mesh_resource::MeshResource,
};

use crate::scene_graph::SceneGraph;

pub fn load_scene_gltf(path: &str, resources: &mut Resources) -> gltf::Result<Vec<SceneGraph>> {
    let (document, buffers, images) = gltf::import(path)?;

    // maps GLTF image index to imported ImageResource
    let mut image_map: HashMap<usize, Arc<CpuResource<TextureImageData>>> = HashMap::new();
    // maps GLTF mesh index to (gltf material index, imported MeshResource)
    let mut mesh_map: HashMap<usize, Vec<(usize, Arc<CpuResource<MeshResource>>)>> = HashMap::new();
    // maps GLTF material index to imported MaterialResource
    let mut material_map: HashMap<usize, Arc<CpuResource<Material>>> = HashMap::new();

    for texture in document.textures() {
        let image_source = &texture.source();
        let image = &images[image_source.index()];
        if !image_map.contains_key(&image_source.index()) {
            let format = match image.format {
                gltf::image::Format::R8 => ash::vk::Format::R8_UNORM,
                gltf::image::Format::R8G8 => ash::vk::Format::R8G8_UNORM,
                gltf::image::Format::R8G8B8 => ash::vk::Format::R8G8B8_UNORM,
                gltf::image::Format::R8G8B8A8 => ash::vk::Format::R8G8B8A8_UNORM,
                gltf::image::Format::B8G8R8 => ash::vk::Format::B8G8R8_UNORM,
                gltf::image::Format::B8G8R8A8 => ash::vk::Format::B8G8R8A8_UNORM,
                gltf::image::Format::R16 => ash::vk::Format::R16_SFLOAT,
                gltf::image::Format::R16G16 => ash::vk::Format::R16G16_SFLOAT,
                gltf::image::Format::R16G16B16 => ash::vk::Format::R16G16B16_SFLOAT,
                gltf::image::Format::R16G16B16A16 => ash::vk::Format::R16G16B16A16_SFLOAT,
            };

            image_map.insert(
                image_source.index(),
                resources.add_texture(
                    &(path.to_string() + "#" + &image_source.index().to_string()),
                    image_source.name().unwrap_or("Untitled"),
                    TextureImageData::new(format, image.width, image.height, &image.pixels),
                ),
            );
        }
    }

    for material in document.materials() {
        let m = material.pbr_metallic_roughness();
        let a = m.base_color_factor();
        let metallic = m.metallic_factor();
        let roughness = m.roughness_factor();
        let emission = material.emissive_factor();
        let mut mat = Material::new(&vec4(a[0], a[1], a[2], a[3]));
        mat.roughness = roughness;
        mat.metallic = metallic;
        mat.emission = vec4(emission[0], emission[1], emission[2], 1.0);

        if let Some(base_color_texture) = m.base_color_texture() {
            mat.albedo_map = Some(
                image_map
                    .get(&base_color_texture.texture().index())
                    .unwrap()
                    .clone(),
            );
        }

        if let Some(metal_roughness_texture) = m.metallic_roughness_texture() {
            mat.metallic_roughness_map = Some(
                image_map
                    .get(&metal_roughness_texture.texture().index())
                    .unwrap()
                    .clone(),
            );
        }

        if let Some(normal_map) = material.normal_texture() {
            mat.normal_map = Some(
                image_map
                    .get(&normal_map.texture().index())
                    .unwrap()
                    .clone(),
            );
        }

        if let Some(emission_texture) = material.emissive_texture() {
            mat.emission_map = Some(
                image_map
                    .get(&emission_texture.texture().index())
                    .unwrap()
                    .clone(),
            );
        }

        material_map.insert(
            material.index().unwrap(),
            resources
                .add_material("", material.name().unwrap_or("Untitled"), mat)
                .clone(),
        );
    }

    for mesh in document.meshes() {
        let imported_meshes = mesh
            .primitives()
            .map(|primitive| {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                let vertices: Vec<Position> = reader
                    .read_positions()
                    .unwrap()
                    .map(|position| Position::new(position[0], position[1], position[2]))
                    .collect();

                let indices: Vec<u32> = if let Some(iter) = reader.read_indices() {
                    iter.into_u32().collect()
                } else {
                    (0..vertices.len() as u32).collect()
                };

                let normals: Vec<nalgebra_glm::Vec3> = if let Some(iter) = reader.read_normals() {
                    iter.map(|normal| nalgebra_glm::Vec3::new(normal[0], normal[1], normal[2]))
                        .collect()
                } else {
                    vec![vec3(0.0, 0.0, 0.0); vertices.len()]
                };

                let tangents: Vec<nalgebra_glm::Vec3> = if let Some(iter) = reader.read_tangents() {
                    iter.map(|tangent| nalgebra_glm::Vec3::new(tangent[0], tangent[1], tangent[2]))
                        .collect()
                } else {
                    vec![vec3(0.0, 0.0, 0.0); vertices.len()]
                };

                let tex_coords: Vec<nalgebra_glm::Vec2> =
                    if let Some(iter) = reader.read_tex_coords(0) {
                        iter.into_f32()
                            .map(|texcoord| nalgebra_glm::Vec2::new(texcoord[0], texcoord[1]))
                            .collect()
                    } else {
                        vec![vec2(0.0, 0.0); vertices.len()]
                    };

                (
                    primitive.material().index().unwrap(),
                    resources
                        .add_mesh(
                            "",
                            mesh.name().unwrap_or("Untitled"),
                            MeshResource::new(indices, vertices, normals, tangents, tex_coords),
                        )
                        .clone(),
                )
            })
            .collect();

        mesh_map.insert(mesh.index(), imported_meshes);
    }

    let mut scene_graph = SceneGraph::new(path);

    for node in document.nodes() {
        let root_node_id = scene_graph.create_node();

        let transform = node.transform().matrix();
        let mut glm_matrix = Mat4::default();
        for (idx, mut column) in glm_matrix.column_iter_mut().enumerate() {
            column[0] = transform[idx][0];
            column[1] = transform[idx][1];
            column[2] = transform[idx][2];
            column[3] = transform[idx][3];
        }
        scene_graph
            .node_mut(root_node_id)
            .with_transform(glm_matrix);

        for child in node.children() {
            scene_graph.node_mut(root_node_id).with_child(child.index());
        }
    }
    
    for node in document.nodes() {
        if let Some(mesh) = node.mesh() {
            let primitives = mesh_map.get(&mesh.index()).unwrap();
            for (material_id, primitive) in primitives {
                let child_id = scene_graph.create_node();
                scene_graph
                    .node_mut(child_id)
                    .with_mesh(primitive.clone())
                    .with_material(material_map.get(material_id).unwrap().clone());
            }
        }
    }

    // for (key, primitives) in &mesh_map {
    //     if primitives.len() > 1 {
    //         let ids = scene_graph.nodes_with_mesh_id(*key);
    //         for id in ids {
    //             scene_graph.expand_node(resources, id, &primitives);
    //         }
    //     } else {
    //         let ids = scene_graph.nodes_with_mesh_id(*key);
    //         for id in ids {
    //             scene_graph
    //                 .node_mut(id)
    //                 .with_mesh(resources.get_mesh_unchecked(primitives[0]));
    //         }
    //     }
    // }

    Ok(vec![scene_graph])
}
