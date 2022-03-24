use nalgebra_glm::vec4;
use renderer::geometry::Vertex;
use renderer::scene::Scene;

fn mul_matrix_array(lhs: &[[f32; 4]; 4], rhs: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut mul = [
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                mul[i][j] += lhs[i][k] * rhs[k][j];
            }
        }
    }

    mul
}

fn load_node(
    mut scene: Scene,
    document: &gltf::Document,
    node: &gltf::Node,
    buffers: &Vec<gltf::buffer::Data>,
    parent_transform: &[[f32; 4]; 4],
) -> Scene {
    // let this_transform = mul_matrix_array(parent_transform, &node.transform().matrix());

    // let mut n = renderer::scene::SceneGraphNode::new(node.name().unwrap_or("Untitled Node"));
    // n.local_transform = node.transform().matrix();
    // n.global_transform = this_transform;

    // if let Some(mesh) = node.mesh() {
    //     for primitive in mesh.primitives() {
    //         let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    //         let vertices: Vec<Vertex> = reader
    //             .read_positions()
    //             .unwrap()
    //             .map(|position| Vertex::new(position[0], position[1], position[2]))
    //             .collect();

    //         let indices: Vec<u32> = if let Some(iter) = reader.read_indices() {
    //             iter.into_u32().collect()
    //         } else {
    //             (0..vertices.len() as u32).collect()
    //         };

    //         let normals: Vec<nalgebra_glm::Vec3> = if let Some(iter) = reader.read_normals() {
    //             iter.map(|normal| nalgebra_glm::Vec3::new(normal[0], normal[1], normal[2]))
    //                 .collect()
    //         } else {
    //             Vec::new()
    //         };

    //         let tangents: Vec<nalgebra_glm::Vec3> = if let Some(iter) = reader.read_tangents() {
    //             iter.map(|tangent| nalgebra_glm::Vec3::new(tangent[0], tangent[1], tangent[2]))
    //                 .collect()
    //         } else {
    //             Vec::new()
    //         };

    //         let tex_coords: Vec<nalgebra_glm::Vec2> = if let Some(iter) = reader.read_tex_coords(0)
    //         {
    //             iter.into_f32()
    //                 .map(|texcoord| nalgebra_glm::Vec2::new(texcoord[0], texcoord[1]))
    //                 .collect()
    //         } else {
    //             (0..vertices.len() as u32)
    //                 .map(|_| nalgebra_glm::Vec2::new(0.0, 0.0))
    //                 .collect()
    //         };

    //         let mut name = mesh.name().unwrap_or("Untitled").to_string();
    //         name.push_str(": ");
    //         name.push_str(&primitive.index().to_string());

    //         let geometry_id =
    //             scene.add_geometry(&name, &indices, &vertices, &normals, &tangents, &tex_coords);

    //         let instance_id = scene.create_instance(geometry_id);
    //         scene.set_matrix(instance_id, &this_transform);
    //         let material = primitive.material();
    //         let m = material.pbr_metallic_roughness();
    //         let a = m.base_color_factor();
    //         let metallic = m.metallic_factor();
    //         let roughness = m.roughness_factor();
    //         let emission = material.emissive_factor();
    //         let mut mat = renderer::scene::Material::new(&vec4(a[0], a[1], a[2], a[3]));
    //         mat.metallic_roughness[0] = roughness;
    //         mat.metallic_roughness[1] = metallic;
    //         mat.emission = vec4(emission[0], emission[1], emission[2], 1.0);

    //         if let Some(base_color_texture) = m.base_color_texture() {
    //             mat.maps[0] = base_color_texture.texture().index() as i32;
    //         }

    //         if let Some(metal_roughness_texture) = m.metallic_roughness_texture() {
    //             mat.maps[1] = metal_roughness_texture.texture().index() as i32;
    //         }

    //         if let Some(emission_texture) = material.emissive_texture() {
    //             mat.maps[3] = emission_texture.texture().index() as i32;
    //         }

    //         scene.add_material(material.name().unwrap_or("Untitled Material"), &mat);
    //     }
    // }

    // for child in node.children() {
    //     n.children.push(child.index())
    // }

    // for child in node.children() {
    //     scene = load_node(scene, document, &child, buffers, &this_transform)
    // }

    //scene.add_node(n);

    scene
}

pub fn load_scene_gltf(path: &str) -> gltf::Result<Scene> {
    let mut new_scene = Scene::new(path);
    let (document, buffers, images) = gltf::import(path)?;
    for image in images {
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
        new_scene.add_image(format, image.width, image.height, &image.pixels);
    }

    for node in document.nodes() {
        let mut n = renderer::scene::SceneGraphNode::new(node.name().unwrap_or("Untitled Node"));
        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                let vertices: Vec<Vertex> = reader
                    .read_positions()
                    .unwrap()
                    .map(|position| Vertex::new(position[0], position[1], position[2]))
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

                let mut name = mesh.name().unwrap_or("Untitled").to_string();
                name.push_str(": ");
                name.push_str(&primitive.index().to_string());

                let geometry_id = new_scene.add_geometry(
                    &name,
                    &indices,
                    &vertices,
                    &normals,
                    &tangents,
                    &tex_coords,
                );

                let instance_id = new_scene.create_instance(geometry_id);
                new_scene.set_matrix(instance_id, &node.transform().matrix());
                let material = primitive.material();
                let m = material.pbr_metallic_roughness();
                let a = m.base_color_factor();
                let metallic = m.metallic_factor();
                let roughness = m.roughness_factor();
                let emission = material.emissive_factor();
                let mut mat = renderer::scene::Material::new(&vec4(a[0], a[1], a[2], a[3]));
                mat.metallic_roughness[0] = roughness;
                mat.metallic_roughness[1] = metallic;
                mat.emission = vec4(emission[0], emission[1], emission[2], 1.0);

                if let Some(base_color_texture) = m.base_color_texture() {
                    mat.maps[0] = base_color_texture.texture().index() as i32;
                }

                if let Some(metal_roughness_texture) = m.metallic_roughness_texture() {
                    mat.maps[1] = metal_roughness_texture.texture().index() as i32;
                }

                if let Some(emission_texture) = material.emissive_texture() {
                    mat.maps[3] = emission_texture.texture().index() as i32;
                }

                new_scene.add_material(material.name().unwrap_or("Untitled Material"), &mat);

                n = n.with_mesh(geometry_id)
            }
        }
        let children: Vec<usize> = node.children().map(|child| child.index()).collect();
        n = n.with_children(&children);
        new_scene.add_node(n);
    }

    for camera in document.cameras() {
        match camera.projection() {
            gltf::camera::Projection::Perspective(cam) => {
                new_scene.add_camera(&renderer::scene::Camera {
                    fov: cam.yfov(),
                    z_near: cam.znear(),
                    z_far: cam.zfar().unwrap_or(1000.0),
                });
            }
            _ => (),
        }
    }

    for scene in document.scenes() {
        if let Some(name) = scene.name() {
            new_scene.name = name.to_string();
        }

        let parent_transform = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        // Build a scene graph
        for node in scene.nodes() {
            new_scene = load_node(new_scene, &document, &node, &buffers, &parent_transform);
        }

        new_scene.nodes[0].name = "Root Node".to_string();
        new_scene.root = 0;
    }

    Ok(new_scene)
}
