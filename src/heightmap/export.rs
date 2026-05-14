use std::{fs::File, io::Write};
use serde::Serialize;
use crate::heightmap::error::GenerationError;

#[derive(Serialize)]
pub struct ExportData {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
    
    pub tile_height_16bit: u16,
    pub total_meshes: usize,
}

pub fn write_export(path: &str, export_data: &ExportData) -> Result<(), GenerationError> {
    let export_data = match serde_json::to_string_pretty(export_data) {
        Ok(d) => d,
        Err(err) => { return Err(GenerationError::FileIO(err.into())); }
    };
    let mut export_data_file = match File::create(path) {
        Ok(f) => f,
        Err(err) => { return Err(GenerationError::FileIO(err)); }
    };

    if let Err(err) = export_data_file.write_all(export_data.as_bytes()) {
        return Err(GenerationError::FileIO(err));
    };

    Ok(())
}
