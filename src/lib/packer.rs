use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub struct Packer;

impl Packer {
    pub fn pack_flat_folder(folder_to_pack: &Path, output_file_path: &Path) -> io::Result<()> {
        if !folder_to_pack.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "la carpeta a empaquetar no existe o no es un directorio",
            ));
        }

        let folder_canonical = folder_to_pack
            .canonicalize()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

        let output_parent = output_file_path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "ruta de salida inválida"))?;

        fs::create_dir_all(output_parent)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

        let output_file = File::create(output_file_path)?;
        let mut zip_writer = ZipWriter::new(output_file);

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        for entry in fs::read_dir(&folder_canonical)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(file_name_os) = path.file_name() {
                    let file_name = file_name_os.to_string_lossy();

                    if file_name.starts_with('.') {
                        continue;
                    }

                    let path_canonical = path
                        .canonicalize()
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

                    if !path_canonical.starts_with(&folder_canonical) {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "ruta de archivo fuera del directorio permitido",
                        ));
                    }

                    let mut file = File::open(&path_canonical)?;
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
        let dir_source = Path::new("test");
        let dir_target = Path::new("test_files/stickermaker.wastickers");

        if let Some(parent) = dir_target.parent() {
            let _ = fs::create_dir_all(parent);
        }

        Packer::pack_flat_folder(dir_source, dir_target)
            .expect("Error crítico: No se pudieron empaquetar los archivos");

        println!("¡Éxito! Archivo generado en: {}", dir_target.display());
    }
}
