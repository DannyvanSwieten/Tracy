use std::{env::temp_dir, fs::File, io::BufReader, path::Path};

use directories::UserDirs;
use renderer::{image_resource::TextureImageData, resources::Resources};

use crate::scene_graph::SceneGraph;

pub struct Project {
    pub folder: String,
    pub name: String,
    pub scene_graph: SceneGraph,
    pub resources: Resources,
}

impl Project {
    pub fn new(name: &str) -> Option<Self> {
        let user_dirs = UserDirs::new();
        if let Some(user_dirs) = user_dirs {
            if let Some(document_dir) = user_dirs.document_dir() {
                let location = document_dir.join("Tracey Projects").join(name);
                match std::fs::create_dir_all(location.to_str().unwrap()) {
                    Err(_) => None,
                    Ok(_) => {
                        let project_file = location.join(name).with_extension("ptrx");
                        File::create(project_file).expect("Failed to write project file");
                        Some(Self {
                            folder: location.to_str().unwrap().to_string(),
                            name: name.to_string(),
                            scene_graph: SceneGraph::new(name),
                            resources: Resources::default(),
                        })
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn tmp() -> Self {
        let dir = temp_dir().join("Tracey").join("Untitled Project");
        if dir.exists() {
            std::fs::remove_dir_all(dir.clone()).expect("Unable to remove file");
        }
        let project_file = dir.clone().join("Untitled Project").with_extension("ptrx");
        std::fs::create_dir_all(dir.clone()).expect("Failed to write project folder");
        let _ = File::create(project_file).expect("Failed to write project file");
        Self {
            folder: dir.to_str().unwrap().to_string(),
            name: "Untitled Project".to_string(),
            scene_graph: SceneGraph::new("Untitled Project"),
            resources: Resources::default(),
        }
    }

    pub fn import(&mut self, path: &str) -> Option<usize> {
        let p = Path::new(path);
        match std::fs::File::open(p) {
            Ok(file) => {
                let reader = BufReader::new(file);
                if let Some(format) = image::ImageFormat::from_extension(p.extension().unwrap()) {
                    let image = image::load(reader, format).expect("Image load failed");
                    Some(
                        self.resources
                            .add_texture(
                                path,
                                p.file_name().unwrap().to_str().unwrap(),
                                TextureImageData::new(
                                    ash::vk::Format::R8G8B8A8_UNORM,
                                    image.width(),
                                    image.height(),
                                    image.as_bytes(),
                                ),
                            )
                            .uid(),
                    )
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}
