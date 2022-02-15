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
    parent_transform: &[[f32; 4]; 4],
) -> Scene {
    let this_transform = mul_matrix_array(parent_transform, &node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        let instance_id = scene.create_instance(mesh.index());
        scene.set_matrix(instance_id, &this_transform)
    }

    if let Some(camera) = node.camera() {
        match camera.projection() {
            gltf::camera::Projection::Perspective(cam) => {
                scene.add_camera(&renderer::scene::Camera {
                    fov: cam.yfov(),
                    z_near: cam.znear(),
                    z_far: cam.zfar().unwrap_or(1000.0),
                })
            }
            _ => (),
        }
        //scene.add_camera(Camera{fov: camera.fov, z_near: camera.z_near, z_far: camera.z_far})
    }

    for child in node.children() {
        scene = load_node(scene, document, &child, &this_transform)
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
        let parent_transform = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        for node in scene.nodes() {
            new_scene = load_node(new_scene, &document, &node, &parent_transform);
        }
    }

    Some(new_scene)
}
