use std::{fs::File, io::{BufWriter, Write}};
use image::{ImageBuffer, Luma};
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
    
    pub metre_16bit: u16,
    pub metre_px: f32,
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

pub struct AdvancedSettings {
    // as percentages
    pub trim_top: f32,
    pub trim_right: f32,
    pub trim_bottom: f32,
    pub trim_left: f32,

    pub override_min_luma: Option<u16>,
    pub override_max_luma: Option<u16>,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        AdvancedSettings {
            trim_top: 0.,
            trim_right: 0.,
            trim_bottom: 0.,
            trim_left: 0.,
            override_max_luma: None,
            override_min_luma: None,
        }
    }
}

pub fn save_heightmap_png(
    path: &str,
    data: &[f32],
    width: u32,
    height: u32,
    grey_per_unit: f32,
    grey_base: f32,
) -> Result<(), GenerationError> {
    let out_width = height;
    let out_height = width;
    let mut img: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::new(out_width, out_height);

    for (i, v) in data.iter().enumerate() {
        let u = if !v.is_finite() {
            32768u16
        } else {
            (grey_base + v * grey_per_unit).clamp(0.0, 65535.0).round() as u16
        };

        let idx = i as u32;
        let x = idx % width;
        let y = idx / width;

        let xf = y;
        let yf = out_height - 1 - x;

        img.put_pixel(xf, yf, Luma([u]));
    }

    let fout = File::create(path).map_err(|e| GenerationError::FileIO(e))?;
    let mut writer = BufWriter::new(fout);
    img.write_to(&mut writer, image::ImageFormat::Png)
        .map_err(|e| GenerationError::ImageWriteError(e))?;

    Ok(())
}
