#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod heightmap;
mod gui;
mod chunk;
mod mesh;
mod math;

fn main() {
    // generate_heightmap("./DashberryData/chunks", "./DashberryData/Assets", 4096).unwrap();
    gui::run().expect("GUI failed");
}
