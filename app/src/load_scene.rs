use std::collections::HashMap;
use std::sync::Arc;

use nalgebra_glm::vec4;
use renderer::cpu_scene::TextureImageData;
use renderer::geometry::Position;
use renderer::mesh_resource::MeshResource;
use renderer::resource::Resource;

use crate::scene_graph::SceneGraph;
use renderer::resources::Resources;

pub fn load_scene_gltf(path: &str, cache: &mut Resources) -> gltf::Result<Vec<SceneGraph>> {
    let (document, buffers, images) = gltf::import(path)?;

    let mut image_map: HashMap<usize, usize> = HashMap::new();
    images.iter().enumerate().for_each(|(index, image)| {
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
        // image_map.insert(
        //     index,
        //     cache.add(TextureImageData::new(
        //         format,
        //         image.width,
        //         image.height,
        //         &image.pixels,
        //     )),
        // );
    });

    let mut mesh_map: HashMap<usize, Vec<Arc<Resource<MeshResource>>>> = HashMap::new();
    let mut material_map: HashMap<usize, usize> = HashMap::new();
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
                    Vec::new()
                };

                let tangents: Vec<nalgebra_glm::Vec3> = if let Some(iter) = reader.read_tangents() {
                    iter.map(|tangent| nalgebra_glm::Vec3::new(tangent[0], tangent[1], tangent[2]))
                        .collect()
                } else {
                    Vec::new()
                };

                let tex_coords: Vec<nalgebra_glm::Vec2> =
                    if let Some(iter) = reader.read_tex_coords(0) {
                        iter.into_f32()
                            .map(|texcoord| nalgebra_glm::Vec2::new(texcoord[0], texcoord[1]))
                            .collect()
                    } else {
                        (0..vertices.len() as u32)
                            .map(|_| nalgebra_glm::Vec2::new(0.0, 0.0))
                            .collect()
                    };

                material_map.insert(primitive.index(), primitive.material().index().unwrap_or(0));
                cache
                    .add(MeshResource::new(indices, vertices, normals))
                    .clone()
            })
            .collect();

        mesh_map.insert(mesh.index(), imported_meshes);
    }

    // for camera in document.cameras() {
    //     match camera.projection() {
    //         gltf::camera::Projection::Perspective(cam) => {
    //             new_scene.add_camera(&renderer::cpu_scene::Camera {
    //                 fov: cam.yfov(),
    //                 z_near: cam.znear(),
    //                 z_far: cam.zfar().unwrap_or(1000.0),
    //             });
    //         }
    //         _ => (),
    //     }
    // }

    let mut scene_graph = SceneGraph::new("");

    for node in document.nodes() {
        let node_id = scene_graph.create_node();
        if let Some(mesh) = node.mesh() {
            scene_graph.node_mut(node_id).with_mesh(Some(mesh.index()));
        }

        let (position, rotation, scale) = node.transform().decomposed();
        scene_graph
            .node_mut(node_id)
            .with_scale(&scale)
            .with_orientation(&rotation)
            .with_position(&position);

        for child in node.children() {
            scene_graph.node_mut(node_id).with_child(child.index());
        }
    }

    for material in document.materials() {
        let m = material.pbr_metallic_roughness();
        let a = m.base_color_factor();
        let metallic = m.metallic_factor();
        let roughness = m.roughness_factor();
        let emission = material.emissive_factor();
        let mut mat = renderer::cpu_scene::Material::new(&vec4(a[0], a[1], a[2], a[3]));
        // mat.metallic_roughness[0] = roughness;
        // mat.metallic_roughness[1] = metallic;
        // mat.emission = vec4(emission[0], emission[1], emission[2], 1.0);

        // if let Some(base_color_texture) = m.base_color_texture() {
        //     mat.maps[0] = *image_map
        //         .get(&base_color_texture.texture().index())
        //         .unwrap() as i32;
        // }

        // if let Some(metal_roughness_texture) = m.metallic_roughness_texture() {
        //     mat.maps[1] = *image_map
        //         .get(&metal_roughness_texture.texture().index())
        //         .unwrap() as i32;
        // }

        // if let Some(normal_map) = material.normal_texture() {
        //     mat.maps[2] = *image_map.get(&normal_map.texture().index()).unwrap() as i32;
        // }

        // if let Some(emission_texture) = material.emissive_texture() {
        //     mat.maps[3] = *image_map.get(&emission_texture.texture().index()).unwrap() as i32;
        // }

        //cache.add(mat);
    }

    // for (key, primitives) in &mesh_map {
    //     if primitives.len() > 1 {
    //         let ids = scene_graph.nodes_with_mesh_id(*key);
    //         for id in ids {
    //             scene_graph.expand_node(id, &primitives);
    //         }
    //     } else {
    //         let ids = scene_graph.nodes_with_mesh_id(*key);
    //         for id in ids {
    //             scene_graph.node_mut(id).with_mesh(Some(primitives[0]));
    //         }
    //     }
    // }

    Ok(vec![scene_graph])
}
