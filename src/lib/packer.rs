use std::fs::{self, File};
use std::io::{self, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub struct Packer;

impl Packer {
    pub fn pack_flat_folder(folder_to_pack: &str, output_file_path: &str) -> io::Result<()> {
        let output_file = File::create(output_file_path)?;
        let mut zip_writer = ZipWriter::new(output_file);

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        for entry in fs::read_dir(folder_to_pack)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(file_name_os) = path.file_name() {
                    let file_name = file_name_os.to_string_lossy();

                    if file_name.starts_with('.') {
                        continue;
                    }

                    let mut file = File::open(&path)?;
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;

                    zip_writer.start_file(file_name.to_string(), options)?;
                    zip_writer.write_all(&buffer)?;
                }
            }
        }

        zip_writer.finish()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_zip() {
        let dir_source = "test";
        let dir_target = "test_files/stickermaker.wastickers";

        if let Some(parent) = std::path::Path::new(dir_target).parent() {
            let _ = fs::create_dir_all(parent);
        }

        Packer::pack_flat_folder(dir_source, dir_target)
            .expect("Error crítico: No se pudieron empaquetar los archivos");

        println!("¡Éxito! Archivo generado en: {}", dir_target);
    }
}
