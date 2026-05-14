#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::{Color32, Ui};
use crate::heightmap::{error::GenerationError, generate_heightmap};

const WINDOW_WIDTH: f32 = 640.;
const WINDOW_HEIGHT: f32 = 520.;
const EXPORT_RESOLUTIONS: [u32; 6] = [512u32, 1024, 2048, 4096, 8192, 16384];

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_max_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

struct MyApp {
    pub assets_directory: Option<String>,
    pub chunks_directory: Option<String>,
    pub resolution: u32,
    pub resolution_slider_value: usize,
    pub export_terrain_map: bool,

    pub generation_error: Option<GenerationError>,
}

impl Default for MyApp {
    fn default() -> Self {
         Self {
             assets_directory: None,
             chunks_directory: None,
             resolution: EXPORT_RESOLUTIONS[3],
             resolution_slider_value: 3,
             generation_error: None,
             export_terrain_map: false,
         }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.visuals_mut().override_text_color = Some(Color32::from_rgb(200, 200, 200));

        let frame_design = egui::containers::Frame {
            inner_margin: egui::epaint::Margin { left: 20, right: 20, top: 20, bottom: 20 },
            fill: egui::Color32::from_rgb(30, 30, 30),
            ..Default::default()
        };

        egui::CentralPanel::default().frame(frame_design).show_inside(ui, |ui| {
            ui.set_pixels_per_point(1.1);

            ui.heading(egui::RichText::new("Heightmap Generator").small());

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("Input");

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Assets directory");
                    add_information(ui, "Select the folder of Fortnite meshes");
                    ui.label(": ");
                    if ui.button("🗀  Select folder").clicked() && let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.assets_directory = Some(path.display().to_string());
                    }
                });

                if self.assets_directory.is_some() {
                    ui.monospace(self.assets_directory.as_ref().unwrap());
                } else {
                    ui.monospace("...");
                }
            });

            ui.add_space(3.0);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Chunks directory");
                    add_information(ui, "Select the folder of randomly-named text files");
                    ui.label(": ");
                    if ui.button("🗀  Select folder").clicked() && let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.chunks_directory = Some(path.display().to_string());
                    }
                });

                if self.chunks_directory.is_some() {
                    ui.monospace(self.chunks_directory.as_ref().unwrap());
                } else {
                    ui.monospace("...");
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("Output");

            ui.horizontal(|ui| {
                ui.label("Image resolution: ");
                ui.add(egui::Slider::new(&mut self.resolution_slider_value, 0..=EXPORT_RESOLUTIONS.len().saturating_sub(1)).show_value(false));
                self.resolution = EXPORT_RESOLUTIONS[self.resolution_slider_value];
                ui.label(format!("{}px{}", self.resolution, if self.resolution > 4096 {" (RAM-heavy)"} else {""}));
            });

            ui.horizontal(|ui| {
                ui.label("Also generate terrain map");
                add_information(ui, "This exports both a normal heightmap, and a heightmap with blank terrain of equal resolution and grey scaling");
                ui.label(": ");
                ui.checkbox(&mut self.export_terrain_map, "");
                ui.label(if self.export_terrain_map { "Yes" } else { "No" });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                let valid = valid_settings(&self);

                if !valid {
                    ui.disable();
                }

                if ui.add_sized([140.0, 40.0], egui::Button::new("Generate")).clicked() {
                    let heightmap_result = generate_heightmap(
                        &self.chunks_directory.as_ref().unwrap(),
                        &self.assets_directory.as_ref().unwrap(),
                        "./out",
                        self.resolution,
                        true
                    );

                    if let Err(err) = heightmap_result {
                        self.generation_error = Some(err);
                    }
                };

                ui.add_space(3.0);

                if self.generation_error.is_some() {
                    ui.colored_label(Color32::from_rgb(235, 200, 200), format!("Generation error: {}", self.generation_error.as_ref().unwrap().to_string()));
                } else if !valid {
                    ui.colored_label(Color32::from_rgb(200, 200, 200), "Invalid settings: missing assets or chunks directory");
                } else {
                    ui.colored_label(Color32::from_rgb(200, 235, 200), "Ready to generate");
                }
            });
        });
    }
}

fn add_information(ui: &mut Ui, text: &str) {
    ui.label(egui::RichText::new("❓").monospace()).on_hover_text(text);
}

fn valid_settings(app: &MyApp) -> bool {
    app.assets_directory.is_some() && app.chunks_directory.is_some()
}
