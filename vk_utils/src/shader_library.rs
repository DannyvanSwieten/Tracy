use ash::version::DeviceV1_0;
use ash::vk::{ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags};
use ash::Device;
use std::collections::HashMap;

use byteorder::ReadBytesExt;
use std::fs::File;

pub fn load_spirv(path: &str) -> Vec<u32> {
    let file = File::open(path).expect(&(String::from("File not found at: ") + &path.to_string()));
    let meta = std::fs::metadata(path).expect("No metadata found for file");
    let mut buf_reader = std::io::BufReader::new(file);

    let mut buffer = vec![0; (meta.len() / 4) as usize];
    buf_reader
        .read_u32_into::<byteorder::NativeEndian>(&mut buffer[..])
        .expect("Failed reading spirv");

    buffer
}

pub struct ShaderLibraryEntry {
    module: ShaderModule,
    stage: ShaderStageFlags,
    entry_point: String,
}
pub struct ShaderLibrary {
    entries: HashMap<String, ShaderLibraryEntry>,
}

impl ShaderLibrary {
    pub fn add_spirv(
        &mut self,
        device: &Device,
        stage: ShaderStageFlags,
        name: &str,
        entry_point: &str,
        code: &[u32],
    ) {
        let info = ShaderModuleCreateInfo::builder().code(code).build();
        let module = unsafe {
            device
                .create_shader_module(&info, None)
                .expect("Shader Module creation failed")
        };

        self.entries.insert(
            String::from(name),
            ShaderLibraryEntry {
                module,
                entry_point: String::from(entry_point),
                stage,
            },
        );
    }
    pub fn add_file(
        &mut self,
        device: &Device,
        stage: ShaderStageFlags,
        name: &str,
        entry_point: &str,
        path: &str,
    ) {
        let spirv = load_spirv(path);
        self.add_spirv(device, stage, name, entry_point, &spirv);
    }
    pub fn get(&self, name: &str) -> Option<&ShaderLibraryEntry> {
        self.entries.get(&String::from(name))
    }
}
