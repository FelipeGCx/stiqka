use chrono::Local;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::models::PackInfo;
use crate::sticker_lib::images::ImageManipulator;
use crate::sticker_lib::packer::Packer;

const MAX_STICKERS: usize = 30;
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif"];

const PACK_FOLDER_NAME: &str = "tmp";
const AUTHOR_FILE_NAME: &str = "author.txt";
const TITLE_FILE_NAME: &str = "title.txt";
const ICON_FILE_NAME: &str = "tray_icon.png";
const STICKER_IMAGE_FORMAT: &str = "webp";

pub fn create_pack(pack_info: PackInfo) -> Result<String, String> {
    let tmp_author_path = PathBuf::from(PACK_FOLDER_NAME).join(AUTHOR_FILE_NAME);
    let tmp_title_path = PathBuf::from(PACK_FOLDER_NAME).join(TITLE_FILE_NAME);
    let tmp_icon_path = PathBuf::from(PACK_FOLDER_NAME).join(ICON_FILE_NAME);

    fs::create_dir_all(PACK_FOLDER_NAME).map_err(|e| e.to_string())?;

    self::write_text_file(tmp_author_path, &pack_info.author_pack).map_err(|e| e.to_string())?;
    self::write_text_file(tmp_title_path, &pack_info.name_pack).map_err(|e| e.to_string())?;

    if let Some(icon_path) = pack_info.icon_path.as_deref() {
        let icon = self::transform_image(icon_path, (96, 96), 50)?;
        icon.save(tmp_icon_path).map_err(|e| e.to_string())?;
    }

    self::process_images(&pack_info.folder_path, pack_info.preserve_images)?;

    Ok(("fff").to_string())
}

fn generate_name(prefix: &str) -> String {
    let date = Local::now().to_string();
    let name = date.replace("-", "").replace(":", "").replace(" ", "");
    let parts: Vec<&str> = name.split('.').collect();

    format!("{}{}{}", prefix, parts[0], &parts[parts.len() - 1][..3])
}

fn write_text_file(filename: PathBuf, sentence: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(sentence.as_bytes())?;
    Ok(())
}

fn transform_image(filename: &str, size: (u32, u32), radius: u32) -> Result<DynamicImage, String> {
    let img_radius = (size.0 as f32 / 100.0 * radius as f32) as u32;
    let mut img = ImageManipulator::load_from_path(filename)?;
    let width = img.width();
    let height = img.height();

    if width < size.0 || height < size.0 {
        let value_resize = if width > height {
            (
                size.0,
                (height as f64 * size.0 as f64 / width as f64) as u32,
            )
        } else {
            (
                (width as f64 * size.0 as f64 / height as f64) as u32,
                size.0,
            )
        };
        img = ImageManipulator::resize(&img, value_resize.0, value_resize.1);
    }

    img = ImageManipulator::thumbnail(&img, size.0, size.1);

    let rounded = ImageManipulator::rounded_border(&img, img_radius);
    let base =
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(size.0, size.1, Rgba([0, 0, 0, 0])));
    let x = (size.0 as i64 - rounded.width() as i64) / 2;
    let y = (size.1 as i64 - rounded.height() as i64) / 2;

    Ok(ImageManipulator::compose(&base, &rounded, x, y))
}

fn process_images(folder: &str, preserve_images: bool) -> Result<(), String> {
    let mut processed_images = Vec::new();

    for entry in fs::read_dir(folder).map_err(|e| e.to_string())? {
        if processed_images.len() >= MAX_STICKERS {
            break;
        }

        let path = entry.map_err(|e| e.to_string())?.path();
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        if !path.is_file() || !IMAGE_EXTENSIONS.contains(&extension) {
            continue;
        }

        if self::is_animated(&path) {
            self::manipulate_animated_image(&path)?;
        } else {
            self::manipulate_static_image(&path)?;
        }

        processed_images.push(path);
    }

    if !preserve_images {
        for image in processed_images {
            fs::remove_file(image).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn is_animated(path: &Path) -> bool {
    path.to_str()
        .and_then(|p| ImageManipulator::is_animated_path(p).ok())
        .unwrap_or(false)
}

fn manipulate_static_image(path: &Path) -> Result<PathBuf, String> {
    let file_name = format!("{}.{}", self::generate_name("WS"), STICKER_IMAGE_FORMAT);
    let output_path = PathBuf::from(PACK_FOLDER_NAME).join(file_name);
    let path_str = path
        .to_str()
        .ok_or_else(|| format!("ruta de imagen inválida: {:?}", path))?;

    let img = self::transform_image(path_str, (512, 512), 8)?;
    img.save(&output_path).map_err(|e| e.to_string())?;

    Ok(output_path)
}

fn manipulate_animated_image(_path: &Path) -> Result<PathBuf, String> {
    // todo!("check gif or webp");
    let file_name = format!("{}.{}", self::generate_name("WS"), STICKER_IMAGE_FORMAT);
    let output_path = PathBuf::from(PACK_FOLDER_NAME).join(file_name);
    Ok(output_path)
}

#[macro_export]
macro_rules! transform_image {
    ($filename:expr) => {
        transform_image($filename, (512, 512), 8)
    };

    ($filename:expr, $size:expr) => {
        transform_image($filename, $size, 8)
    };

    ($filename:expr, $size:expr, $radius:expr) => {
        transform_image($filename, $size, $radius)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_name() {
        let prefix = "WSP";

        let name_generated = generate_name(prefix);

        println!("¡Éxito! texto generado: {}", name_generated);
    }

    #[test]
    fn test_write_file() {
        let filename = "tmp/text.txt";
        let sentence = "pruebas";

        write_text_file(filename.into(), sentence).unwrap();

        println!("¡Éxito! Archivo generado en: {}", filename);
    }

    #[test]
    fn test_extension() {
        let folder_test = "tmp";
        for entry in fs::read_dir(folder_test).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            println!("{:?}", path.extension());
        }
    }
    #[test]
    fn test_create_pack() {
        let pack_info = PackInfo {
            name_pack: "Prueba".to_owned(),
            author_pack: "FelipeGCx".to_owned(),
            folder_path: r"assets/test".to_owned(),
            icon_path: Some(r"assets/test/icon.png".to_owned()),
            preserve_images: true,
        };

        let result = create_pack(pack_info);

        assert!(result.is_ok(), "create_pack falló: {:?}", result);
        assert!(PathBuf::from(PACK_FOLDER_NAME)
            .join(AUTHOR_FILE_NAME)
            .exists());
        assert!(PathBuf::from(PACK_FOLDER_NAME)
            .join(TITLE_FILE_NAME)
            .exists());
        assert!(PathBuf::from(PACK_FOLDER_NAME)
            .join(ICON_FILE_NAME)
            .exists());
    }
}
