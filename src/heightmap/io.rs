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
    vmin: f32,
    vmax: f32,
    settings: &AdvancedSettings,
) -> Result<(), GenerationError> {
    let trim_top = settings.trim_top.clamp(0.0, 100.0) * 0.01;
    let trim_right = settings.trim_right.clamp(0.0, 100.0) * 0.01;
    let trim_bottom = settings.trim_bottom.clamp(0.0, 100.0) * 0.01;
    let trim_left = settings.trim_left.clamp(0.0, 100.0) * 0.01;

    let crop_x0 = (width as f32 * trim_left).round() as u32;
    let crop_x1 = (width as f32 * trim_right).round() as u32;
    let crop_y0 = (height as f32 * trim_top).round() as u32;
    let crop_y1 = (height as f32 * trim_bottom).round() as u32;

    let crop_w = width.saturating_sub(crop_x0 + crop_x1).max(1);
    let crop_h = height.saturating_sub(crop_y0 + crop_y1).max(1);

    let mut crop_vmin = f32::INFINITY;
    let mut crop_vmax = f32::NEG_INFINITY;

    for y in 0..crop_h {
        for x in 0..crop_w {
            let src_x = crop_x0 + x;
            let src_y = crop_y0 + y;
            let i = (src_y * width + src_x) as usize;
            let v = data[i];
            if v.is_finite() {
                crop_vmin = crop_vmin.min(v);
                crop_vmax = crop_vmax.max(v);
            }
        }
    }

    if !crop_vmin.is_finite() || !crop_vmax.is_finite() {
        crop_vmin = vmin;
        crop_vmax = vmax;
    }

    let out_width = crop_h.max(1);
    let out_height = crop_w.max(1);
    let mut img: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::new(out_width, out_height);

    for y in 0..crop_h {
        for x in 0..crop_w {
            let src_x = crop_x0 + x;
            let src_y = crop_y0 + y;
            let i = (src_y * width + src_x) as usize;
            let v = data[i];

            let u = if !v.is_finite() || (crop_vmin == crop_vmax) {
                32768u16
            } else {
                let t = ((v - crop_vmin) / (crop_vmax - crop_vmin)).clamp(0.0, 1.0);
                if let (Some(lo), Some(hi)) = (settings.override_min_luma, settings.override_max_luma) {
                    let lo = lo as f32;
                    let hi = hi as f32;
                    (lo + t * (hi - lo)).round() as u16
                } else {
                    (t * 65535.0).round() as u16
                }
            };

            let xf = crop_h - 1 - y;
            let yf = crop_w - 1 - x;
            img.put_pixel(xf, yf, Luma([u]));
        }
    }

    let fout = File::create(path).map_err(GenerationError::FileIO)?;
    let mut writer = BufWriter::new(fout);
    img.write_to(&mut writer, image::ImageFormat::Png)
        .map_err(GenerationError::ImageWriteError)?;

    Ok(())
}
