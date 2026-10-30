use image::{DynamicImage, ImageBuffer, GenericImageView, ImageFormat};

pub struct ImageManipulator;

impl ImageManipulator {
    pub fn load_from_bytes(data: &[u8]) -> Result<DynamicImage, String> {
        image::load_from_memory(data).map_err(|e| e.to_string())
    }

    pub fn load_from_path(path: &str) -> Result<DynamicImage, String> {
        image::open(path).map_err(|e| e.to_string())
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
    }
}