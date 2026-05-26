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

pub fn save_heightmap_png(
    path: &str,
    data: &[f32],
    width: u32,
    height: u32,
    vmin: f32,
    vmax: f32,
) -> Result<(), GenerationError> {
    let out_width = height;
    let out_height = width;
    let mut img: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::new(out_width, out_height);

    for (i, v) in data.iter().enumerate() {
        let v = *v;
        let u = if !v.is_finite() || (vmin == vmax) {
            32768u16
        } else {
            let t = ((v - vmin) / (vmax - vmin)).clamp(0.0, 1.0);
            (t * 65535.0).round() as u16
        };

        let idx = i as u32;
        let x = idx % width;
        let y = idx / width;

        let xf = out_width - 1 - y;
        let yf = out_height - 1 - x;

        img.put_pixel(xf, yf, Luma([u]));
    }

    let fout = File::create(path).map_err(|e| GenerationError::FileIO(e))?;
    let mut writer = BufWriter::new(fout);
    img.write_to(&mut writer, image::ImageFormat::Png)
        .map_err(|e| GenerationError::ImageWriteError(e))?;

    Ok(())
}

