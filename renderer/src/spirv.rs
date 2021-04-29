use byteorder::ReadBytesExt;
use std::fs::File;

pub fn load_spirv(path: &str) -> Vec<u32> {
    let file = File::open(path).expect("File not found");
    let meta = std::fs::metadata(path).expect("No metadata found for file");
    let mut buf_reader = std::io::BufReader::new(file);

    let mut buffer = vec![0; (meta.len() / 4) as usize];
    buf_reader
        .read_u32_into::<byteorder::NativeEndian>(&mut buffer[..])
        .expect("Failed reading spirv");

    buffer
}
