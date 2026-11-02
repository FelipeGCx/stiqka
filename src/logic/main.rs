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
const PACK_EXTENSION: &str = "wastickers";

pub fn create_pack(
    pack_info: PackInfo,
    on_progress: Option<Box<dyn Fn(f32) + Send + Sync>>,
) -> Result<String, String> {
    let report = |p: f32| {
        if let Some(cb) = &on_progress {
            cb(p.clamp(0.0, 1.0));
        }
    };

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

    report(0.10);

    let processed_images = if let Some(cb) = &on_progress {
        self::process_images(&pack_info.folder_path, Some(&|p: f32| cb(0.10 + p * 0.70)))?
    } else {
        self::process_images(&pack_info.folder_path, None)?
    };

    let preserve = pack_info.preserve_images;

    self::packer(pack_info)?;

    report(0.90);

    fs::remove_dir_all(PACK_FOLDER_NAME).map_err(|e| e.to_string())?;

    if !preserve {
        for image in processed_images {
            fs::remove_file(image).map_err(|e| e.to_string())?;
        }
    }

    report(1.0);
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
    let img = ImageManipulator::load_from_path(filename)?;

    Ok(self::transform_dynamic_image(img, size, radius))
}

fn transform_dynamic_image(mut img: DynamicImage, size: (u32, u32), radius: u32) -> DynamicImage {
    let img_radius = (size.0 as f32 / 100.0 * radius as f32) as u32;
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

    ImageManipulator::compose(&base, &rounded, x, y)
}

fn process_images(
    folder: &str,
    on_progress: Option<&dyn Fn(f32)>,
) -> Result<Vec<PathBuf>, String> {
    let entries: Vec<_> = fs::read_dir(folder)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect();

    let total_valid: usize = entries
        .iter()
        .filter_map(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_owned())
        })
        .filter(|ext| IMAGE_EXTENSIONS.contains(&ext.as_str()))
        .count()
        .min(MAX_STICKERS);

    if total_valid == 0 {
        return Ok(Vec::new());
    }

    let mut processed_images = Vec::new();
    let mut processed_count: usize = 0;

    for entry in entries {
        if processed_images.len() >= MAX_STICKERS {
            break;
        }

        let path = entry.path();
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
        processed_count += 1;

        let p = processed_count as f32 / total_valid as f32;
        if let Some(cb) = on_progress {
            cb(p);
        }
    }

    Ok(processed_images)
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

fn manipulate_animated_image(path: &Path) -> Result<PathBuf, String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| format!("ruta de imagen inválida: {:?}", path))?;
    let file_name = format!("{}.{}", self::generate_name("WS"), STICKER_IMAGE_FORMAT);
    let output_path = PathBuf::from(PACK_FOLDER_NAME).join(file_name);

    let frames = ImageManipulator::load_frames(path_str)?;
    let transformed_frames: Vec<_> = frames
        .into_iter()
        .map(|(frame, delay_ms)| {
            let image =
                self::transform_dynamic_image(DynamicImage::ImageRgba8(frame), (512, 512), 8);
            (image.to_rgba8(), delay_ms)
        })
        .collect();

    let standardized = ImageManipulator::standardize_animated_frames(transformed_frames);

    let output_path_str = output_path
        .to_str()
        .ok_or_else(|| "ruta de salida inválida".to_string())?;

    ImageManipulator::encode_animated_webp(&standardized, output_path_str, 60.0)?;

    Ok(output_path)
}

fn packer(pack_info: PackInfo) -> Result<(), String> {
    let folder_to_pack = PathBuf::from(PACK_FOLDER_NAME);
    let filename = format!("{}.{}", pack_info.name_pack, PACK_EXTENSION);
    let output_file_path = PathBuf::from(pack_info.folder_path).join(filename);

    Packer::pack_flat_folder(&folder_to_pack, &output_file_path).map_err(|e| e.to_string())
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

        let result = create_pack(pack_info, None);

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
