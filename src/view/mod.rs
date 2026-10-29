use eframe::egui;
use egui::{FontId, RichText};
use image::GenericImageView;
use rfd::FileDialog;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::translations::t;
use crate::logic::controller::Controller;
use crate::models::PackInfo;

pub struct SfwaApp {
    name_pack: String,
    author_pack: String,
    folder_path: String,
    icon_pack: Option<egui::TextureHandle>,
    icon_path: Option<String>,
    selected_image: Option<egui::TextureHandle>,
    preserve_images: bool,
    pack_creator: Arc<Mutex<Controller>>,
    show_toast: bool,
    toast_message: String,
    toast_is_error: bool,
    toast_timer: f32,
}

impl SfwaApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();

        let variants = [
            (
                "Montserrat-Regular",
                include_bytes!("../../assets/fonts/Montserrat/static/Montserrat-Regular.ttf")
                    .as_slice(),
            ),
            (
                "Montserrat-Medium",
                include_bytes!("../../assets/fonts/Montserrat/static/Montserrat-Medium.ttf")
                    .as_slice(),
            ),
            (
                "Montserrat-SemiBold",
                include_bytes!("../../assets/fonts/Montserrat/static/Montserrat-SemiBold.ttf")
                    .as_slice(),
            ),
            (
                "Montserrat-Bold",
                include_bytes!("../../assets/fonts/Montserrat/static/Montserrat-Bold.ttf").as_slice(),
            ),
            (
                "Montserrat-Black",
                include_bytes!("../../assets/fonts/Montserrat/static/Montserrat-Black.ttf").as_slice(),
            ),
        ];

        for (name, bytes) in variants {
            fonts
                .font_data
                .insert(name.to_owned(), egui::FontData::from_static(bytes).into());

            fonts
                .families
                .insert(egui::FontFamily::Name(name.into()), vec![name.to_owned()]);
        }

        fonts.families.insert(
            egui::FontFamily::Proportional,
            vec!["Montserrat-Regular".to_owned()],
        );

        cc.egui_ctx.set_fonts(fonts);

        Self {
            name_pack: String::new(),
            author_pack: String::new(),
            folder_path: String::new(),
            icon_pack: None,
            icon_path: None,
            selected_image: None,
            preserve_images: true,
            pack_creator: Arc::new(Mutex::new(Controller::new())),
            show_toast: false,
            toast_message: String::new(),
            toast_is_error: false,
            toast_timer: 0.0,
        }
    }

    fn load_image_from_path(
        ctx: &egui::Context,
        path: &std::path::Path,
    ) -> Option<egui::TextureHandle> {
        let mut image = image::open(path).ok()?;
        let (w, h) = image.dimensions();
        let size = w.min(h);
        let x = (w - size) / 2;
        let y = (h - size) / 2;
        image = image.crop_imm(x, y, size, size);
        let image = image.to_rgba8();
        let size = [size as usize, size as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &image);
        let texture_name = format!("selected_image_{}", path.display());
        Some(ctx.load_texture(texture_name, color_image, egui::TextureOptions::default()))
    }

    fn is_form_valid(&self) -> bool {
        !self.name_pack.trim().is_empty()
            && !self.author_pack.trim().is_empty()
            && !self.folder_path.trim().is_empty()
            && self.selected_image.is_some()
    }

    fn show_toast(&mut self, message: String, is_error: bool) {
        self.toast_message = message;
        self.toast_is_error = is_error;
        self.show_toast = true;
        self.toast_timer = 3.0;
    }
}

impl eframe::App for SfwaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        if self.show_toast {
            self.toast_timer -= ctx.input(|i| i.stable_dt);
            if self.toast_timer <= 0.0 {
                self.show_toast = false;
            }
        }

        let pack_creator = self.pack_creator.clone();
        let mut creator = pack_creator.lock().unwrap();
        
        let is_processing = creator.is_processing();
        let current_progress = creator.progress();
        
        if is_processing {
            creator.update_progress(current_progress + 0.01);
        }

        if let Some(result) = creator.try_take_result() {
            match result {
                Ok(_path) => {
                    self.show_toast(t("successCreate"), false);
                }
                Err(e) => {
                    self.show_toast(e, true);
                }
            }
        }
        drop(creator);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let components_height = 24.0;
                let horizontal_gap = 6.0;
                let components_width = 300.0;

                ui.add_space(20.0);
                ui.add_sized(
                    [components_width, components_height],
                    egui::TextEdit::singleline(&mut self.name_pack).hint_text(t("name_pack")),
                );

                ui.add_space(10.0);
                ui.add_sized(
                    [components_width, components_height],
                    egui::TextEdit::singleline(&mut self.author_pack).hint_text(t("name_author")),
                );

                let available_width = ui.available_width();
                let left_padding = ((available_width - components_width) / 2.0).max(0.0);
                let input_folder_width =
                    (components_width - horizontal_gap - components_height).max(0.0);

                ui.add_space(20.0);
                ui.label(RichText::new(t("max_images")).font(FontId::proportional(10.0)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(left_padding);

                    ui.add_sized(
                        [input_folder_width, components_height],
                        egui::TextEdit::singleline(&mut self.folder_path)
                            .hint_text("Ruta de la carpeta"),
                    );

                    if ui
                        .add_sized(
                            [components_height, components_height],
                            egui::Button::new("..."),
                        )
                        .clicked()
                    {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.folder_path = path.display().to_string();
                        }
                    }
                });

                ui.add_space(20.0);

                let size = egui::vec2(120.0, 120.0);
                let corner_radius = egui::CornerRadius::same(255);

                let response = if let Some(texture) = &self.selected_image {
                    ui.add(
                        egui::Image::new(texture)
                            .fit_to_exact_size(size)
                            .corner_radius(corner_radius)
                            .sense(egui::Sense::click()),
                    )
                } else if let Some(texture) = &self.icon_pack {
                    ui.add(
                        egui::Image::new(texture)
                            .fit_to_exact_size(size)
                            .corner_radius(corner_radius)
                            .sense(egui::Sense::click()),
                    )
                } else {
                    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
                    let painter = ui.painter();
                    painter.rect_filled(rect, corner_radius, egui::Color32::from_gray(60));
                    painter.circle_filled(rect.center(), 30.0, egui::Color32::from_gray(120));
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "+",
                        egui::FontId::proportional(48.0),
                        egui::Color32::from_gray(180),
                    );
                    response
                };

                if response.clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Imágenes", &["png", "jpg", "jpeg", "webp", "bmp"])
                        .pick_file()
                    {
                        self.icon_path = Some(path.display().to_string());
                        self.selected_image = Self::load_image_from_path(ctx, &path);
                    }
                }

                ui.add_space(15.0);
                ui.checkbox(&mut self.preserve_images, t("preserve_images"));

                let visuals = ui.visuals();
                let (bg_color, text_color) = if visuals.dark_mode {
                    (
                        egui::Color32::from_rgb(0, 175, 156),
                        egui::Color32::from_rgb(0, 13, 20),
                    )
                } else {
                    (
                        egui::Color32::from_rgb(129, 199, 132),
                        egui::Color32::from_rgb(0, 0, 0),
                    )
                };

                ui.add_space(10.0);

                let button_enabled = self.is_form_valid() && !is_processing;

                let button = egui::Button::new(
                    RichText::new(if is_processing { t("creating") } else { t("create_pack") })
                        .font(FontId::new(
                            14.0,
                            egui::FontFamily::Name("Montserrat-Medium".into()),
                        ))
                        .color(text_color)
                        .strong(),
                )
                .fill(if button_enabled { bg_color } else { bg_color.gamma_multiply(0.5) })
                .corner_radius(12.0)
                .min_size(egui::vec2(160.0, 36.0));

                let button_response = ui.add_enabled(button_enabled, button);

                if is_processing {
                    ui.add_space(10.0);
                    ui.add(egui::ProgressBar::new(current_progress)
                        .desired_width(160.0)
                        .show_percentage()
                        .animate(true));
                }

                if button_response.clicked() && button_enabled {
                    let pack_info = PackInfo {
                        name_pack: self.name_pack.clone(),
                        author_pack: self.author_pack.clone(),
                        folder_path: self.folder_path.clone(),
                        icon_path: self.icon_path.clone(),
                        preserve_images: self.preserve_images,
                    };

                    let pack_creator = self.pack_creator.clone();
                    thread::spawn(move || {
                        let mut c = pack_creator.lock().unwrap();
                        c.start(pack_info);
                    });
                }

                ui.add_space(26.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(RichText::new("version 1.0.0").font(FontId::proportional(10.0)));
                });
            });
        });

        if self.show_toast {
            egui::Window::new("")
                .fixed_pos(egui::pos2(
                    ctx.screen_rect().width() - 320.0,
                    ctx.screen_rect().height() - 100.0,
                ))
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .frame(egui::Frame::window(&ctx.style()).fill(
                    if self.toast_is_error {
                        eframe::egui::Color32::from_rgb(220, 50, 50)
                    } else {
                        eframe::egui::Color32::from_rgb(50, 180, 50)
                    }
                ))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(15.0);
                        ui.label(RichText::new(&self.toast_message)
                            .color(eframe::egui::Color32::WHITE)
                            .font(FontId::proportional(14.0)));
                        ui.add_space(15.0);
                    });
                    ui.add_space(10.0);
                });
        }
    }
}