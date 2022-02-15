use renderer::geometry::Vertex;
use renderer::scene::Scene;

fn dot_array(lhs: &[f32; 4], rhs: &[f32; 4]) -> f32 {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2] + lhs[3] * rhs[3]
}

// fn mul_matrix_array(lhs: &[[f32; 4]; 4], rhs: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {

// }

fn load_node(
    mut scene: Scene,
    document: &gltf::Document,
    node: &gltf::Node,
    parent_transform: &gltf::scene::Transform,
) -> Scene {
    if let Some(mesh) = node.mesh() {
        let instance_id = scene.create_instance(mesh.index());
        match node.transform() {
            gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => {
                scene.set_position_values(
                    instance_id,
                    translation[0],
                    translation[1],
                    translation[2],
                );
                // scene.set_orientation(instance_id, tr.rotation);
                scene.set_scale_values(instance_id, scale[0], scale[1], scale[2]);
            }
            gltf::scene::Transform::Matrix { matrix } => scene.set_matrix(instance_id, &matrix),
        }
    }

    for child in node.children() {
        scene = load_node(scene, document, &child, &node.transform())
    }

    scene
}

pub fn load_scene(path: &str) -> Option<Scene> {
    let mut new_scene = Scene::new();
    let (document, buffers, images) = gltf::import(path).expect("GLTF import failed");
    for mesh in document.meshes() {
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

            new_scene.add_geometry(&indices, &vertices);
        }
    }

    for image in images {
        let format = match image.format {
            gltf::image::Format::R8 => ash::vk::Format::R8_UNORM,
            gltf::image::Format::R8G8 => ash::vk::Format::R16G16_SINT,
            gltf::image::Format::R8G8B8 => ash::vk::Format::R8G8B8_UNORM,
            gltf::image::Format::R8G8B8A8 => ash::vk::Format::R8G8B8A8_UNORM,
            gltf::image::Format::B8G8R8 => ash::vk::Format::B8G8R8_UNORM,
            gltf::image::Format::B8G8R8A8 => ash::vk::Format::B8G8R8A8_UNORM,
            gltf::image::Format::R16 => ash::vk::Format::R16_SINT,
            gltf::image::Format::R16G16 => ash::vk::Format::R16G16_SINT,
            gltf::image::Format::R16G16B16 => ash::vk::Format::R16G16B16_SINT,
            gltf::image::Format::R16G16B16A16 => ash::vk::Format::R16G16B16A16_SINT,
        };
        new_scene.add_image(format, image.width, image.height, &image.pixels);
    }

    for scene in document.scenes() {
        for node in scene.nodes() {
            new_scene = load_node(new_scene, &document, &node, &node.transform());
        }
    }

    Some(new_scene)
}
