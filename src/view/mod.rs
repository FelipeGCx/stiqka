use crate::logic::controller::Controller;
use crate::models::PackInfo;
use crate::translations::{set_language, t, Language};
use directories::ProjectDirs;
use eframe::egui;
use egui::{FontId, RichText};
use image::GenericImageView;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Serialize, Deserialize)]
struct Preferences {
    language: String,
}

impl Preferences {
    fn path() -> Option<PathBuf> {
        ProjectDirs::from("com", "stiqka", "Stiqka")
            .map(|proj| proj.config_dir().join("preferences.json"))
    }

    fn load() -> Self {
        if let Some(path) = Self::path() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(prefs) = serde_json::from_str::<Preferences>(&data) {
                    return prefs;
                }
            }
        }
        Preferences {
            language: "en".to_string(),
        }
    }

    fn save(&self) {
        if let Some(path) = Self::path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(&path, json);
            }
        }
    }
}

pub struct StiqkaApp {
    name_pack: String,
    author_pack: String,
    folder_path: String,
    icon_pack: Option<egui::TextureHandle>,
    icon_path: Option<String>,
    selected_image: Option<egui::TextureHandle>,
    preserve_images: bool,
    language: Language,
    show_lang_menu: bool,
    pack_creator: Arc<Mutex<Controller>>,
    show_toast: bool,
    toast_message: String,
    toast_is_error: bool,
    toast_timer: f32,
    flag_textures: HashMap<Language, Option<egui::TextureHandle>>,
}

impl StiqkaApp {
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
                "NotoSansJP-Regular",
                include_bytes!("../../assets/fonts/NotoSansJP/static/NotoSansJP-Regular.ttf")
                    .as_slice(),
            ),
            (
                "NotoSansJP-Medium",
                include_bytes!("../../assets/fonts/NotoSansJP/static/NotoSansJP-Medium.ttf")
                    .as_slice(),
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
            vec![
                "Montserrat-Regular".to_owned(),
                "NotoSansJP-Regular".to_owned(),
            ],
        );

        fonts.families.insert(
            egui::FontFamily::Name("Medium".into()),
            vec![
                "Montserrat-Medium".to_owned(),
                "NotoSansJP-Regular".to_owned(),
            ],
        );

        cc.egui_ctx.set_fonts(fonts);

        let prefs = Preferences::load();
        let language = Language::from_code(&prefs.language).unwrap_or(Language::Es);
        set_language(language);

        let mut flag_textures = HashMap::new();
        for lang in Language::all() {
            flag_textures.insert(lang, None);
        }

        Self {
            name_pack: String::new(),
            author_pack: String::new(),
            folder_path: String::new(),
            icon_pack: None,
            icon_path: None,
            selected_image: None,
            preserve_images: true,
            language,
            show_lang_menu: false,
            pack_creator: Arc::new(Mutex::new(Controller::new())),
            show_toast: false,
            toast_message: String::new(),
            toast_is_error: false,
            toast_timer: 0.0,
            flag_textures,
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

    fn load_flag_texture(
        ctx: &egui::Context,
        bytes: &[u8],
        name: &str,
    ) -> Option<egui::TextureHandle> {
        let image = image::load_from_memory(bytes).ok()?;
        let (w, h) = image.dimensions();
        let size = w.min(h);
        let x = (w - size) / 2;
        let y = (h - size) / 2;
        let image = image.crop_imm(x, y, size, size).to_rgba8();
        let size = [size as usize, size as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &image);
        Some(ctx.load_texture(name, color_image, egui::TextureOptions::default()))
    }

    fn flag_texture(&mut self, ctx: &egui::Context, lang: Language) -> Option<egui::TextureHandle> {
        if let Some(Some(cached)) = self.flag_textures.get(&lang).cloned() {
            return Some(cached);
        }
        let bytes: &[u8] = match lang {
            Language::En => include_bytes!("../../assets/flags/gb.png") as &[u8],
            Language::Es => include_bytes!("../../assets/flags/es.png") as &[u8],
            Language::Fr => include_bytes!("../../assets/flags/fr.png") as &[u8],
            Language::De => include_bytes!("../../assets/flags/de.png") as &[u8],
            Language::It => include_bytes!("../../assets/flags/it.png") as &[u8],
            Language::Pt => include_bytes!("../../assets/flags/pt.png") as &[u8],
            Language::Jp => include_bytes!("../../assets/flags/jp.png") as &[u8],
        };
        let texture = Self::load_flag_texture(ctx, bytes, lang.flag_file());
        self.flag_textures.insert(lang, texture.clone());
        texture
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

impl eframe::App for StiqkaApp {
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

        if is_processing {
            if let Some(p) = creator.try_recv_progress() {
                creator.update_progress(p);
            }
        }

        if let Some(result) = creator.try_take_result() {
            match result {
                Ok(_path) => {
                    self.show_toast(t("successCreate"), false);
                    self.name_pack.clear();
                    self.author_pack.clear();
                    self.folder_path.clear();
                    self.icon_path = None;
                    self.selected_image = None;
                    self.icon_pack = None;
                }
                Err(e) => {
                    self.show_toast(e, true);
                }
            }
        }

        let progress = creator.progress();
        drop(creator);

        for lang in Language::all() {
            let _ = Self::flag_texture(self, ctx, lang);
        }

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
                            .hint_text(t("folder_path")),
                    );

                    if ui
                        .add_sized(
                            [components_height, components_height],
                            egui::Button::new(t("browse_folder")),
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
                    RichText::new(if is_processing {
                        t("creating")
                    } else {
                        t("create_pack")
                    })
                    .font(FontId::new(
                        14.0,
                        egui::FontFamily::Name("Medium".into()),
                    ))
                    .color(text_color)
                    .strong(),
                )
                .fill(if button_enabled {
                    bg_color
                } else {
                    bg_color.gamma_multiply(0.5)
                })
                .corner_radius(12.0)
                .min_size(egui::vec2(160.0, 36.0));

                let button_response = ui.add_enabled(button_enabled, button);

                if is_processing {
                    ui.add_space(10.0);
                    ui.add(
                        egui::ProgressBar::new(progress)
                            .desired_width(160.0)
                            .show_percentage()
                            .animate(true),
                    );
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

                ui.horizontal(|ui| {
                    ui.add_space(left_padding);

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                        let current_lang = self.language;

                        ui.horizontal(|ui| {
                            let lang_button = egui::Button::new(
                                RichText::new(current_lang.name()).font(FontId::proportional(10.0)),
                            );
                            let lang_response = ui.add(lang_button);

                            if lang_response.clicked() {
                                self.show_lang_menu = !self.show_lang_menu;
                            }

                            if self.show_lang_menu {
                                let rect = lang_response.rect;
                                let window_width = 150.0;
                                let item_height = 26.0;
                                let window_height = (item_height * 6.0) + 16.0;
                                let pos = egui::pos2(
                                    (rect.left().max(4.0) - 16.0).max(0.0),
                                    ((rect.top() - window_height - 4.0).max(4.0) + 34.0).max(0.0),
                                );

                                let window_response = egui::Window::new(t("language"))
                                    .fixed_pos(pos)
                                    .fixed_size([window_width, window_height])
                                    .collapsible(false)
                                    .resizable(false)
                                    .title_bar(false)
                                    .show(ctx, |ui| {
                                        for lang in Language::all() {
                                            ui.horizontal(|ui| {
                                                if let Some(tex) = self
                                                    .flag_textures
                                                    .get(&lang)
                                                    .and_then(|o| o.clone())
                                                {
                                                    ui.add(
                                                        egui::Image::new(&tex)
                                                            .fit_to_exact_size(egui::vec2(
                                                                16.0, 16.0,
                                                            ))
                                                            .corner_radius(
                                                                egui::CornerRadius::same(4),
                                                            ),
                                                    );
                                                }
                                                let selected = self.language == lang;
                                                if ui
                                                    .selectable_label(selected, lang.name())
                                                    .clicked()
                                                {
                                                    self.language = lang;
                                                    set_language(lang);
                                                    Preferences {
                                                        language: lang.code().to_string(),
                                                    }
                                                    .save();
                                                    self.show_lang_menu = false;
                                                }
                                            });
                                        }
                                    });

                                if let Some(inner) = window_response {
                                    let win_rect = inner.response.rect;
                                    if ctx.input(|i| i.pointer.any_click()) {
                                        let click_pos = ctx
                                            .input(|i| i.pointer.hover_pos())
                                            .unwrap_or_default();
                                        if !win_rect.contains(click_pos)
                                            && !lang_response.rect.contains(click_pos)
                                        {
                                            self.show_lang_menu = false;
                                        }
                                    }
                                }
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} {}",
                                t("app_version"),
                                env!("CARGO_PKG_VERSION")
                            ))
                            .font(FontId::proportional(10.0)),
                        );
                    });
                });
            });
        });

        if self.show_toast {
            let screen_width = ctx.screen_rect().width();
            let toast_width = (screen_width - 20.0).max(200.0);

            egui::Window::new("")
                .fixed_pos(egui::pos2(10.0, ctx.screen_rect().height() - 100.0))
                .default_width(toast_width)
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(if self.toast_is_error {
                            eframe::egui::Color32::from_rgb(220, 50, 50)
                        } else {
                            eframe::egui::Color32::from_rgb(0, 150, 136)
                        })
                        .outer_margin(egui::Margin::same(10)),
                )
                .show(ctx, |ui| {
                    ui.set_max_width(toast_width - 20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(&self.toast_message)
                                .color(eframe::egui::Color32::WHITE)
                                .font(FontId::proportional(14.0)),
                        );
                    });
                });
        }
    }
}
