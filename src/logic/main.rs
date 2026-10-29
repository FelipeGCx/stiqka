use chrono::Local;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::models::PackInfo;
use crate::sticker_lib::images::ImageManipulator;
use crate::sticker_lib::packer::Packer;

const MAX_STICKERS: usize = 30;
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "gif"];

const AUTHOR_FILE_NAME: &str = "temp/author.txt";
const TITLE_FILE_NAME: &str = "temp/title.txt";
const ICON_FILE_NAME: &str = "temp/tray_icon.png";
const STICKER_IMAGE_FORMAT: &str = "webp";

pub fn create_pack(pack_info: PackInfo) -> Result<String, String> {
    // let folder_name = self::generate_name("WSP");

    self::write_text_file(AUTHOR_FILE_NAME, &pack_info.author_pack).map_err(|e| e.to_string())?;
    self::write_text_file(TITLE_FILE_NAME, &pack_info.name_pack).map_err(|e| e.to_string())?;

    if let Some(icon_path) = pack_info.icon_path.as_deref() {
        let icon = self::transform_image(icon_path, (96, 96), 16)?;
        fs::create_dir_all("temp").map_err(|e| e.to_string())?;
        icon.save(ICON_FILE_NAME).map_err(|e| e.to_string())?;
    }

    self::process_images(&pack_info.folder_path);

    Ok(("fff").to_string())
}

fn generate_name(prefix: &str) -> String {
    let date = Local::now().to_string();
    let name = date.replace("-", "").replace(":", "").replace(" ", "");
    let parts: Vec<&str> = name.split('.').collect();

    format!("{}{}{}", prefix, parts[0], &parts[parts.len() - 1][..3])
}

fn write_text_file(filename: &str, sentence: &str) -> std::io::Result<()> {
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

fn process_images(folder: &str) {
    let mut processed_images = Vec::new();
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if path.is_file() && IMAGE_EXTENSIONS.contains(&extension) {
            let _file_name = format!("{}.{}", self::generate_name("WS"), STICKER_IMAGE_FORMAT);
            // TODO process image logic
            processed_images.push(path);
        }
        if processed_images.len() > MAX_STICKERS {
            break;
        }
    }
}

fn manipulate_image(path: PathBuf){
    if path.extension() === "webp" {

    }
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
        let filename = "temp/text.txt";
        let sentence = "pruebas";

        write_text_file(filename, sentence);

        println!("¡Éxito! Archivo generado en: {}", filename);
    }

    #[test]
    fn test_extension() {
        let folder_test = "temp";
        for entry in fs::read_dir(folder_test).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            println!("{:?}", path.extension());
        }
    }
}
