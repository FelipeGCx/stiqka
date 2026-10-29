mod translations;
mod view;
mod logic;
mod models;

#[path = "lib/mod.rs"]
#[allow(dead_code)]
mod sticker_lib;

use eframe::egui;
use image;
use view::SfwaApp;

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("No se pudo cargar el icono")
        .into_rgba8();

    let (width, height) = image.dimensions();

    let icon = egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(icon)
            .with_always_on_top()
            .with_resizable(false)
            .with_inner_size([340.0, 430.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Sticker Fast WA",
        opts,
        Box::new(|cc| Ok(Box::new(SfwaApp::new(cc)))),
    )
}