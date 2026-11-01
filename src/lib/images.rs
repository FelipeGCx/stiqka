use std::io::Cursor;
use std::ops::Deref;

use webp::{AnimEncoder, AnimFrame, WebPConfig};
use image::codecs::gif::GifDecoder;
use image::codecs::webp::WebPDecoder;
use image::{
    AnimationDecoder, DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgba, RgbaImage,
};

pub struct ImageManipulator;

impl ImageManipulator {
    pub fn is_gif(data: &[u8]) -> bool {
        data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a")
    }

    pub fn is_webp(data: &[u8]) -> bool {
        data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP"
    }

    pub fn is_animated_gif(data: &[u8]) -> Result<bool, String> {
        let decoder = GifDecoder::new(Cursor::new(data)).map_err(|e| e.to_string())?;
        Ok(decoder.into_frames().take(2).count() > 1)
    }

    pub fn is_animated_webp(data: &[u8]) -> Result<bool, String> {
        let decoder = WebPDecoder::new(Cursor::new(data)).map_err(|e| e.to_string())?;
        Ok(decoder.has_animation())
    }

    pub fn is_animated(data: &[u8]) -> Result<bool, String> {
        if Self::is_gif(data) {
            Self::is_animated_gif(data)
        } else if Self::is_webp(data) {
            Self::is_animated_webp(data)
        } else {
            Ok(false)
        }
    }

    pub fn is_animated_path(path: &str) -> Result<bool, String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        Self::is_animated(&data)
    }

    pub fn load_from_bytes(data: &[u8]) -> Result<DynamicImage, String> {
        image::load_from_memory(data).map_err(|e| e.to_string())
    }

    pub fn load_from_path(path: &str) -> Result<DynamicImage, String> {
        image::open(path).map_err(|e| e.to_string())
    }

    pub fn load_frames(path: &str) -> Result<Vec<(RgbaImage, u32)>, String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;

        let frames = if Self::is_gif(&data) {
            GifDecoder::new(Cursor::new(data))
                .map_err(|e| e.to_string())?
                .into_frames()
                .collect_frames()
                .map_err(|e| e.to_string())?
        } else if Self::is_webp(&data) {
            WebPDecoder::new(Cursor::new(data))
                .map_err(|e| e.to_string())?
                .into_frames()
                .collect_frames()
                .map_err(|e| e.to_string())?
        } else {
            return Err(format!("formato animado no soportado: {path}"));
        };

        Ok(frames
            .into_iter()
            .map(|frame| {
                let (numer, denom) = frame.delay().numer_denom_ms();
                let delay_ms = if denom == 0 { 0 } else { numer / denom };
                (frame.into_buffer(), delay_ms)
            })
            .collect())
    }

    pub const MAX_ANIMATED_FRAMES: usize = 50;
    pub const MAX_ANIMATED_DURATION_MS: u32 = 3500;
    pub const MIN_FRAME_DELAY_MS: u32 = 42; // 1000 / 24 fps
    pub const MAX_FRAME_DELAY_MS: u32 = 67; // 1000 / 15 fps

    /// Estandariza frames animados para cumplir límites de sticker:
    /// - máximo 50 fotogramas
    /// - duración máxima 3500 ms
    /// - delay por frame entre 42 ms (24 fps) y 67 ms (15 fps)
    pub fn standardize_animated_frames(
        frames: Vec<(RgbaImage, u32)>,
    ) -> Vec<(RgbaImage, u32)> {
        if frames.is_empty() {
            return frames;
        }

        // 1) Máximo 50 frames: muestreo uniforme si hay más
        let frames = if frames.len() > Self::MAX_ANIMATED_FRAMES {
            let step = frames.len() as f64 / Self::MAX_ANIMATED_FRAMES as f64;
            (0..Self::MAX_ANIMATED_FRAMES)
                .map(|i| {
                    let idx = (i as f64 * step) as usize;
                    frames[idx.min(frames.len() - 1)].clone()
                })
                .collect()
        } else {
            frames
        };

        // 2) Clampear cada delay al rango permitido [42, 67] ms
        let mut adjusted: Vec<(RgbaImage, u32)> = frames
            .into_iter()
            .map(|(img, d)| {
                let d = d.clamp(Self::MIN_FRAME_DELAY_MS, Self::MAX_FRAME_DELAY_MS);
                (img, d)
            })
            .collect();

        // 3) Si la duración total excede 3500 ms, escalar proporcionalmente
        let total: u32 = adjusted.iter().map(|(_, d)| d).sum();
        if total > Self::MAX_ANIMATED_DURATION_MS {
            let scale = Self::MAX_ANIMATED_DURATION_MS as f64 / total as f64;
            adjusted = adjusted
                .into_iter()
                .map(|(img, d)| {
                    let nd = (d as f64 * scale) as u32;
                    let nd = nd.clamp(Self::MIN_FRAME_DELAY_MS, Self::MAX_FRAME_DELAY_MS);
                    (img, nd)
                })
                .collect();

            // 4) Si aún excede, eliminar frames del final hasta que quepa
            let total: u32 = adjusted.iter().map(|(_, d)| d).sum();
            if total > Self::MAX_ANIMATED_DURATION_MS {
                let mut remaining = Self::MAX_ANIMATED_DURATION_MS;
                let mut result = Vec::new();
                for (img, d) in adjusted {
                    if remaining >= d {
                        result.push((img, d));
                        remaining -= d;
                    } else {
                        break;
                    }
                }
                adjusted = result;
            }
        }

        adjusted
    }

    pub fn encode_animated_webp(
        frames: &[(RgbaImage, u32)],
        output_path: &str,
        quality: f32,
    ) -> Result<(), String> {
        const MAX_SIZE: usize = 500_000;
        let quality = quality.clamp(1.0, 100.0);

        let (width, height) = frames
            .first()
            .map(|(frame, _)| frame.dimensions())
            .ok_or_else(|| "la animación no tiene frames".to_string())?;

        if frames
            .iter()
            .any(|(frame, _)| frame.dimensions() != (width, height))
        {
            return Err("todos los frames deben tener el mismo tamaño".to_string());
        }

        // Se respeta el tamaño original (p. ej. 512×512). Nunca se escala.
        // Compresión lossy real con libwebp (VP8), probando calidades
        // descendentes hasta que quepa bajo 500 KB.
        let qualities: &[f32] = &[60.0, 50.0, 40.0, 30.0, 20.0, 10.0];
        let mut best: Option<Vec<u8>> = None;

        for &q in qualities {
            if q > quality {
                continue;
            }

            let mut config = WebPConfig::new().map_err(|_| "WebPConfig init failed".to_string())?;
            config.lossless = 0;
            config.alpha_compression = 1;
            config.quality = q;

            let mut encoder = AnimEncoder::new(width, height, &config);

            for (frame, delay_ms) in frames {
                let anim_frame = AnimFrame::from_rgba(
                    frame.as_raw(),
                    width,
                    height,
                    *delay_ms as i32,
                );
                encoder.add_frame(anim_frame);
            }

            let webp_memory = encoder.encode();
            let bytes: Vec<u8> = webp_memory.deref().to_vec();

            if bytes.len() <= MAX_SIZE {
                return std::fs::write(output_path, &bytes).map_err(|e| e.to_string());
            }

            if best.is_none() || bytes.len() < best.as_ref().unwrap().len() {
                best = Some(bytes);
            }
        }

        if let Some(bytes) = best {
            return std::fs::write(output_path, &bytes).map_err(|e| e.to_string());
        }

        Err("no se pudo comprimir la animación por debajo de 500 KB sin escalar".to_string())
    }

    pub fn resize(
        image: &DynamicImage,
        width: u32,
        height: u32,
    ) -> DynamicImage {
        let resized = image::imageops::resize(image, width, height, image::imageops::Lanczos3);
        DynamicImage::ImageRgba8(resized)
    }

    pub fn thumbnail(image: &DynamicImage, max_width: u32, max_height: u32) -> DynamicImage {
        let width = image.width();
        let height = image.height();
        let scale = (max_width as f64 / width as f64).min(max_height as f64 / height as f64);

        if scale >= 1.0 {
            return image.clone();
        }

        let new_width = ((width as f64 * scale).round() as u32).max(1);
        let new_height = ((height as f64 * scale).round() as u32).max(1);

        Self::resize(image, new_width, new_height)
    }

    pub fn crop(
        image: &DynamicImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> DynamicImage {
        let mut result_buf = ImageBuffer::new(width, height);
        for row in 0..height {
            for col in 0..width {
                let src_x = x + col;
                let src_y = y + row;
                let pixel = image.get_pixel(src_x, src_y);
                result_buf.put_pixel(col, row, pixel);
            }
        }
        DynamicImage::ImageRgba8(result_buf)
    }

    pub fn grayscale(image: &DynamicImage) -> DynamicImage {
        let mut buf = image.to_luma8();
        for pixel in buf.pixels_mut() {
            let l = pixel.0[0];
            pixel.0 = [(((0.299 * l as f32 + 0.587 * l as f32 + 0.114 * l as f32)).round()) as u8];
        }
        DynamicImage::ImageLuma8(buf)
    }

    pub fn flip_horizontal(image: &DynamicImage) -> DynamicImage {
        let buf = image.to_rgba8();
        DynamicImage::ImageRgba8(image::imageops::flip_horizontal(&buf))
    }

    pub fn flip_vertical(image: &DynamicImage) -> DynamicImage {
        let buf = image.to_rgba8();
        DynamicImage::ImageRgba8(image::imageops::flip_vertical(&buf))
    }

    pub fn rotate(image: &DynamicImage, angle: f32) -> DynamicImage {
        let rot_count = ((angle / (std::f32::consts::PI * 2.0)) * 4.0).round() as i32;
        let mut result = image.clone();
        for _ in 0..rot_count {
            result = image::DynamicImage::ImageRgba8(image::imageops::rotate90(
                &result.to_rgba8()
            ));
        }
        result
    }

    pub fn blur(image: &DynamicImage, radius: f32) -> DynamicImage {
        let buf = image.to_rgba8();
        DynamicImage::ImageRgba8(image::imageops::blur(&buf, radius))
    }

    pub fn compose(
        base: &DynamicImage,
        overlay: &DynamicImage,
        offset_x: i64,
        offset_y: i64,
    ) -> DynamicImage {
        let mut result = base.to_rgba8();
        let overlay_rgba = overlay.to_rgba8();
        
        image::imageops::overlay(&mut result, &overlay_rgba, offset_x, offset_y);
        DynamicImage::ImageRgba8(result)
    }

    pub fn rounded_border(image: &DynamicImage, radius: u32) -> DynamicImage {
        let mut buf = image.to_rgba8();
        let width = buf.width();
        let height = buf.height();
        let radius = radius.min(width.min(height) / 2);
        if radius == 0 {
            return DynamicImage::ImageRgba8(buf);
        }

        const SAMPLES: u32 = 4;
        let total_samples = (SAMPLES * SAMPLES) as f32;
        let r = radius as f32;

        for y in 0..height {
            for x in 0..width {
                let mut covered = 0u32;
                for sy in 0..SAMPLES {
                    for sx in 0..SAMPLES {
                        let px = x as f32 + (sx as f32 + 0.5) / SAMPLES as f32;
                        let py = y as f32 + (sy as f32 + 0.5) / SAMPLES as f32;
                        if Self::inside_rounded_rect(px, py, width as f32, height as f32, r) {
                            covered += 1;
                        }
                    }
                }

                if covered < SAMPLES * SAMPLES {
                    let factor = covered as f32 / total_samples;
                    let pixel = buf.get_pixel_mut(x, y);
                    pixel.0[3] = (pixel.0[3] as f32 * factor).round() as u8;
                }
            }
        }

        DynamicImage::ImageRgba8(buf)
    }

    fn inside_rounded_rect(px: f32, py: f32, width: f32, height: f32, radius: f32) -> bool {
        let cx = px.clamp(radius, width - radius);
        let cy = py.clamp(radius, height - radius);
        let dx = px - cx;
        let dy = py - cy;
        dx * dx + dy * dy <= radius * radius
    }

    pub fn dimensions(image: &DynamicImage) -> (u32, u32) {
        (image.width(), image.height())
    }

    pub fn format_name(format: ImageFormat) -> &'static str {
        match format {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpeg => "JPEG",
            ImageFormat::WebP => "WEBP",
            ImageFormat::Gif => "GIF",
            ImageFormat::Bmp => "BMP",
            ImageFormat::Ico => "ICO",
            _ => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_bytes() {
        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/1.webp")).unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    #[test]
    fn test_resize() {
        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/2.png")).unwrap();
        let resized = ImageManipulator::resize(&img, 512, 512);
        assert_eq!(resized.width(), 512);
        assert_eq!(resized.height(), 512);
    }

    #[test]
    fn test_thumbnail() {
        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/3.jpg")).unwrap();
        let (width, height) = (img.width(), img.height());

        let thumb = ImageManipulator::thumbnail(&img, 24, 12);
        assert!(thumb.width() <= 24);
        assert!(thumb.height() <= 12);

        let untouched = ImageManipulator::thumbnail(&img, width, height);
        assert_eq!((untouched.width(), untouched.height()), (width, height));
    }

    #[test]
    fn test_crop() {
        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/3.jpg")).unwrap();
        // Image is 48x48, crop 32x32 from (0,0)
        let cropped = ImageManipulator::crop(&img, 0, 0, 32, 32);
        assert_eq!(cropped.width(), 32);
        assert_eq!(cropped.height(), 32);
    }

    #[test]
    fn test_grayscale() {
        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/img_test.jpg")).unwrap();
        let gray = ImageManipulator::grayscale(&img);
        assert!(matches!(gray, DynamicImage::ImageLuma8(_)));
    }

    #[test]
    fn test_flip_horizontal() {
        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/img_test.jpg")).unwrap();
        let flipped = ImageManipulator::flip_horizontal(&img);
        assert!(matches!(flipped, DynamicImage::ImageRgba8(_)));
    }

    #[test]
    fn test_flip_vertical() {
        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/img_test.jpg")).unwrap();
        let flipped = ImageManipulator::flip_vertical(&img);
        assert!(matches!(flipped, DynamicImage::ImageRgba8(_)));
    }

    #[test]
    fn test_rotate() {
        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/img_test.jpg")).unwrap();
        let rotated = ImageManipulator::rotate(&img, 90.0);
        assert!(rotated.width() > 0);
        assert!(rotated.height() > 0);
    }

    #[test]
    fn test_blur() {
        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/img_test.jpg")).unwrap();
        let blurred = ImageManipulator::blur(&img, 5.0);
        assert!(blurred.width() > 0);
        assert!(blurred.height() > 0);
    }

    #[test]
    fn test_rounded_border() {
        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/img_test.jpg")).unwrap();
        let rounded = ImageManipulator::rounded_border(&img, 8);
        assert_eq!(rounded.width(), img.width());
        assert_eq!(rounded.height(), img.height());

        let rgba = rounded.to_rgba8();
        assert_eq!(rgba.get_pixel(0, 0).0[3], 0);
        assert_eq!(rgba.get_pixel(img.width() / 2, img.height() / 2).0[3], 255);
    }

    #[test]
    fn test_generate_outputs() {
        use std::fs;

        let out_dir = "test_files";
        fs::create_dir_all(out_dir).unwrap();

        let img = ImageManipulator::load_from_bytes(include_bytes!("../../assets/test/img_test.jpg")).unwrap();
        let icon = ImageManipulator::load_from_bytes(include_bytes!("../../assets/icon.png")).unwrap();

        let outputs: [(&str, DynamicImage); 9] = [
            ("resize.png", ImageManipulator::resize(&img, 96, 96)),
            ("crop.png", ImageManipulator::crop(&img, 8, 8, 32, 32)),
            ("grayscale.png", ImageManipulator::grayscale(&img)),
            ("flip_horizontal.png", ImageManipulator::flip_horizontal(&img)),
            ("flip_vertical.png", ImageManipulator::flip_vertical(&img)),
            ("rotate.png", ImageManipulator::rotate(&img, 90.0)),
            ("blur.png", ImageManipulator::blur(&img, 5.0)),
            ("compose.png", ImageManipulator::compose(&img, &icon, 4, 4)),
            ("rounded_border.png", ImageManipulator::rounded_border(&img, 64)),
        ];

        for (name, image) in outputs {
            image.save(format!("{out_dir}/{name}")).unwrap();
        }
    }

    #[test]
    fn test_format_name() {
        assert_eq!(ImageManipulator::format_name(ImageFormat::Png), "PNG");
        assert_eq!(ImageManipulator::format_name(ImageFormat::Jpeg), "JPEG");
        assert_eq!(ImageManipulator::format_name(ImageFormat::Gif), "GIF");
    }

    #[test]
    fn test_detect_gif() {
        let data = include_bytes!("../../assets/test/4_gf.gif");
        assert!(ImageManipulator::is_gif(data));
        assert!(!ImageManipulator::is_webp(data));
        assert!(ImageManipulator::is_animated_gif(data).unwrap());
        assert!(ImageManipulator::is_animated(data).unwrap());
    }

    #[test]
    fn test_detect_animated_webp() {
        let data = include_bytes!("../../assets/test/5_wp.webp");
        assert!(ImageManipulator::is_webp(data));
        assert!(!ImageManipulator::is_gif(data));
        assert!(ImageManipulator::is_animated_webp(data).unwrap());
        assert!(ImageManipulator::is_animated(data).unwrap());
    }

    #[test]
    fn test_static_images_are_not_animated() {
        let png = include_bytes!("../../assets/test/2.png");
        let jpg = include_bytes!("../../assets/test/3.jpg");
        let webp = include_bytes!("../../assets/test/1.webp");

        assert!(!ImageManipulator::is_gif(png));
        assert!(!ImageManipulator::is_webp(png));
        assert!(!ImageManipulator::is_animated(png).unwrap());

        assert!(!ImageManipulator::is_gif(jpg));
        assert!(!ImageManipulator::is_animated(jpg).unwrap());

        assert!(ImageManipulator::is_webp(webp));
        assert!(!ImageManipulator::is_animated(webp).unwrap());
    }

    #[test]
    fn test_encode_animated_webp() {
        std::fs::create_dir_all("test_files").unwrap();

        for (source, output) in [
            ("assets/test/4_gf.gif", "test_files/animated_gif.webp"),
            ("assets/test/5_wp.webp", "test_files/animated_webp.webp"),
        ] {
            let frames = ImageManipulator::load_frames(source).unwrap();
            assert!(frames.len() > 1);

            ImageManipulator::encode_animated_webp(&frames, output, 60.0).unwrap();

            let data = std::fs::read(output).unwrap();
            let size_kb = data.len() as f64 / 1024.0;
            let decoder = WebPDecoder::new(Cursor::new(data)).unwrap();
            assert!(decoder.has_animation());

            let decoded = decoder.into_frames().collect_frames().unwrap();
            assert_eq!(decoded.len(), frames.len());
            let (dw, dh) = decoded[0].buffer().dimensions();
            let (sw, sh) = frames[0].0.dimensions();
            assert!(dw > 0 && dh > 0);
            assert_eq!(dw, sw, "dimensiones cambiaron: {dw}x{dh} vs {sw}x{sh}");
            assert_eq!(dh, sh, "dimensiones cambiaron: {dw}x{dh} vs {sw}x{sh}");
            assert!(size_kb < 500.0, "animación supera 500 KB: {:.1} KB", size_kb);
        }
    }

    #[test]
    fn test_standardize_animated_frames() {
        // Crea 100 frames con delays alternados muy pequeños y muy grandes
        let mut frames: Vec<(RgbaImage, u32)> = Vec::new();
        for i in 0..100u32 {
            let img = RgbaImage::from_pixel(10, 10, Rgba([i as u8, 0, 0, 255]));
            let delay = if i % 2 == 0 { 10 } else { 200 }; // fuera de rango [42, 67]
            frames.push((img, delay));
        }

        let standardized = ImageManipulator::standardize_animated_frames(frames);

        // Máximo 50 frames
        assert!(standardized.len() <= ImageManipulator::MAX_ANIMATED_FRAMES);

        // Cada delay dentro del rango permitido
        for (_, d) in &standardized {
            assert!(
                *d >= ImageManipulator::MIN_FRAME_DELAY_MS,
                "delay {} menor al mínimo", d
            );
            assert!(
                *d <= ImageManipulator::MAX_FRAME_DELAY_MS,
                "delay {} mayor al máximo", d
            );
        }

        // Duración total <= 3500 ms
        let total: u32 = standardized.iter().map(|(_, d)| d).sum();
        assert!(
            total <= ImageManipulator::MAX_ANIMATED_DURATION_MS,
            "duración {} ms excede 3500 ms", total
        );
    }

    #[test]
    fn test_standardize_animated_frames_duration_reduction() {
        // 50 frames con delay 67 ms = 3350 ms (bajo 3500, no se toca)
        let mut frames: Vec<(RgbaImage, u32)> = Vec::new();
        for i in 0..50u32 {
            let img = RgbaImage::from_pixel(10, 10, Rgba([i as u8, 0, 0, 255]));
            frames.push((img, 67));
        }

        let standardized = ImageManipulator::standardize_animated_frames(frames);
        assert_eq!(standardized.len(), 50);
        let total: u32 = standardized.iter().map(|(_, d)| d).sum();
        assert_eq!(total, 3350);
        assert!(total <= ImageManipulator::MAX_ANIMATED_DURATION_MS);
    }
}