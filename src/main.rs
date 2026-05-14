mod heightmap;
mod gui;
mod chunk;

fn main() {
    // generate_heightmap("./DashberryData/chunks", "./DashberryData/Assets", 4096).unwrap();
    gui::run().expect("GUI failed");
}
