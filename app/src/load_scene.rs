use nalgebra_glm::{vec3, vec4};
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
    let this_transform = mul_matrix_array(parent_transform, &node.transform().matrix());
    let mut n = renderer::scene::SceneGraphNode::default();
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

            let tex_coords: Vec<nalgebra_glm::Vec2> = if let Some(iter) = reader.read_tex_coords(0)
            {
                iter.into_f32()
                    .map(|texcoord| nalgebra_glm::Vec2::new(texcoord[0], texcoord[1]))
                    .collect()
            } else {
                (0..vertices.len() as u32)
                    .map(|_| nalgebra_glm::Vec2::new(0.0, 0.0))
                    .collect()
            };

            let geometry_id = scene.add_geometry(&indices, &vertices, &tex_coords);
            let instance_id = scene.create_instance(geometry_id);
            scene.set_matrix(instance_id, &this_transform);
            let m = primitive.material().pbr_metallic_roughness();
            let a = m.base_color_factor();
            let metallic = m.metallic_factor();
            let roughness = m.roughness_factor();
            let emission = primitive.material().emissive_factor();
            scene.set_material_metallic(instance_id, metallic);
            scene.set_material_roughness(instance_id, roughness);
            scene.set_material_base_color(instance_id, &vec4(a[0], a[1], a[2], a[3]));
            scene.set_material_emission(
                instance_id,
                &vec3(emission[0], emission[1], emission[2]),
                1.0,
            );
            if let Some(base_color_texture) = m.base_color_texture() {
                scene.set_material_base_color_texture(
                    instance_id,
                    base_color_texture.texture().index(),
                );
            }

            if let Some(metal_roughness_texture) = m.metallic_roughness_texture() {
                scene.set_material_metallic_roughness_texture(
                    instance_id,
                    metal_roughness_texture.texture().index(),
                );
            }

            if let Some(emission_texture) = primitive.material().emissive_texture() {
                scene.set_material_emission_texture(instance_id, emission_texture.texture().index())
            }

            n.mesh = Some(instance_id);
        }
    }

    if let Some(camera) = node.camera() {
        match camera.projection() {
            gltf::camera::Projection::Perspective(cam) => {
                scene.add_camera(&renderer::scene::Camera {
                    fov: cam.yfov(),
                    z_near: cam.znear(),
                    z_far: cam.zfar().unwrap_or(1000.0),
                });
            }
            _ => (),
        }

        n.camera = Some(camera.index());
    }

    for child in node.children() {
        n.children.push(child.index());
        scene = load_node(scene, document, &child, buffers, &this_transform)
    }

    scene.add_node(n);

    scene
}

pub fn load_scene(path: &str) -> gltf::Result<Scene> {
    let mut new_scene = Scene::default();
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

    for scene in document.scenes() {
        new_scene.name = scene.name().unwrap_or("Untitled").to_string();
        let parent_transform = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        for node in scene.nodes() {
            new_scene = load_node(new_scene, &document, &node, &buffers, &parent_transform);
        }
    }

    Ok(new_scene)
}
