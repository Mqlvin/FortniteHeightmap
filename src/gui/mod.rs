#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::Visuals;

use crate::heightmap::{error::GenerationError, generate_heightmap};

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 240.0]) // wide enough for the drag-drop overlay text
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Default)]
struct MyApp {
    assets_directory: Option<String>,
    chunks_directory: Option<String>,
    resolution: u32,

    generation_error: Option<GenerationError>,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {

        let frame_design = egui::containers::Frame {
            inner_margin: egui::epaint::Margin { left: 20, right: 20, top: 20, bottom: 20 },
            fill: egui::Color32::from_rgb(230, 225, 220),
            ..Default::default()
        };

        egui::CentralPanel::default().frame(frame_design).show_inside(ui, |ui| {
            ui.set_pixels_per_point(1.1);
            ui.set_visuals(Visuals::light()); // light mode

            ui.heading("Fornite Heightmap Generator");

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            if self.generation_error.is_some() {
                ui.label("Generation error: ");
                ui.label(self.generation_error.as_ref().unwrap().to_string());
                return;
            }

            ui.horizontal(|ui| {
                ui.label("Assets directory: ");

                if self.assets_directory.is_some() {
                    ui.monospace(self.assets_directory.as_ref().unwrap());
                    return;
                }

                if ui.button("Select folder…").clicked() && let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.assets_directory = Some(path.display().to_string());
                }
                
            });

            ui.horizontal(|ui| {
                ui.label("Chunks directory: ");

                if self.chunks_directory.is_some() {
                    ui.monospace(self.chunks_directory.as_ref().unwrap());
                    return;
                }

                if ui.button("Select folder…").clicked() && let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.chunks_directory = Some(path.display().to_string());
                }
            });

            ui.horizontal(|ui| {
                ui.label("Resolution: ");
                ui.add(egui::Slider::new(&mut self.resolution, 128..=8192).step_by(128.)); // better step?
            });

            ui.horizontal(|ui| {
                if self.assets_directory.is_none() || self.chunks_directory.is_none() {
                    ui.label("Select asset and chunk directory to get started");
                    ui.disable();
                    return;
                }

                ui.label("Generate heightmap: ");
                if ui.add_sized([60.0, 32.0], egui::Button::new("Generate")).clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        self.generation_error = match generate_heightmap(
                            &self.chunks_directory.as_ref().unwrap(),
                            &self.assets_directory.as_ref().unwrap(),
                            &path.to_str().expect("A save directory"),
                            self.resolution
                        ) {
                            Ok(()) => None,
                            Err(err) => Some(err)
                        };
                    }
                };
            });
            
        });
    }
}
