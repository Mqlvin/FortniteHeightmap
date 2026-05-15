use image::{ImageBuffer, Luma};
use rayon::prelude::*;
use std::{fs::File, io::BufWriter, sync::Mutex};

use crate::heightmap::{error::GenerationError, math::edge};

pub fn rasterize_heightmap(
    vertices: &[[f32; 3]],
    faces: &[[usize; 3]],
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    z_min: f32,
    z_max: f32,
    width: u32,
    height: u32
) -> Result<Vec<f32>, GenerationError> {
    let x_span = x_max - x_min;
    let y_span = y_max - y_min;
    let z_span = z_max - z_min;

    if x_span == 0.0 || y_span == 0.0 || z_span == 0.0 {
        return Err(GenerationError::MapRasterizationError("Bounds must have non-zero extent".to_string()));
    }

    let scale_x = (width as f32 - 1.0) / x_span;
    let scale_y = (height as f32 - 1.0) / y_span;
    let scale = scale_x.max(scale_y); // max to scale image up
    let out_w = x_span * scale;
    let out_h = y_span * scale;
    let pad_x = (width as f32 - out_w) * 0.5;
    let pad_y = (height as f32 - out_h) * 0.5;

    let hm = Mutex::new(vec![f32::NAN; (width * height) as usize]);
    let eps = 1e-7f32;

    faces.par_iter().for_each(|face| {
        let v0 = vertices[face[0]];
        let v1 = vertices[face[1]];
        let v2 = vertices[face[2]];

        let to_screen = |v: [f32; 3]| -> [f32; 3] {
            // use unified scale and center offsets (pad_x/pad_y)
            let sx = (v[0] - x_min) * scale + pad_x;
            let sy = (v[1] - y_min) * scale + pad_y;
            let rz = (v[2] - z_min) / z_span; // not clamped
            [sx, sy, rz]
        };

        let p0 = to_screen(v0);
        let p1 = to_screen(v1);
        let p2 = to_screen(v2);

        // axis-aligned integer bounds from float coords
        let min_x = p0[0].min(p1[0]).min(p2[0]).floor().max(0.0) as i32;
        let max_x = p0[0].max(p1[0]).max(p2[0]).ceil().min((width as f32) - 1.0) as i32;
        let min_y = p0[1].min(p1[1]).min(p2[1]).floor().max(0.0) as i32;
        let max_y = p0[1].max(p1[1]).max(p2[1]).ceil().min((height as f32) - 1.0) as i32;

        // quick reject if bbox empty
        if min_x > max_x || min_y > max_y {
            return;
        }

        // local edge function using float coords
        let a = [p0[0], p0[1]];
        let b = [p1[0], p1[1]];
        let c = [p2[0], p2[1]];
        let area = edge(a, b, c);
        if area.abs() < 1e-12 {
            return;
        }
        let ccw = area > 0.0;

        let is_top_left = |ax: f32, ay: f32, bx: f32, by: f32| -> bool {
            if (ay - by).abs() <= eps {
                return ax < bx;
            }
            ay < by
        };

        let mut local_updates: Vec<(usize, f32)> = Vec::new();

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                // sample at pixel center
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                let w0 = edge([b[0], b[1]], [c[0], c[1]], [px, py]);
                let w1 = edge([c[0], c[1]], [a[0], a[1]], [px, py]);
                let w2 = edge([a[0], a[1]], [b[0], b[1]], [px, py]);

                // fast reject depending on winding
                if ccw {
                    if w0 < -eps || w1 < -eps || w2 < -eps {
                        continue;
                    }
                } else {
                    if w0 > eps || w1 > eps || w2 > eps {
                        continue;
                    }
                }

                let mut on_edge_reject = false;
                if w0.abs() <= eps {
                    if !is_top_left(b[0], b[1], c[0], c[1]) {
                        on_edge_reject = true;
                    }
                }
                if on_edge_reject { continue; }
                if w1.abs() <= eps {
                    if !is_top_left(c[0], c[1], a[0], a[1]) {
                        on_edge_reject = true;
                    }
                }
                if on_edge_reject { continue; }
                if w2.abs() <= eps {
                    if !is_top_left(a[0], a[1], b[0], b[1]) {
                        on_edge_reject = true;
                    }
                }
                if on_edge_reject { continue; }

                // compute barycentrics from float edge values and area
                let l0 = w0 / area;
                let l1 = w1 / area;
                let l2 = w2 / area;

                let z = l0 * p0[2] + l1 * p1[2] + l2 * p2[2];

                let idx = (y as u32 * width + x as u32) as usize;
                local_updates.push((idx, z));
            }
        }

        let mut hm_guard = hm.lock().unwrap();
        for (idx, z) in local_updates {
            match hm_guard[idx] {
                v if v.is_nan() => hm_guard[idx] = z,
                old if z > old => hm_guard[idx] = z,
                _ => {}
            }
        }
    });

    let mut hm = hm.into_inner().unwrap();
    for v in hm.iter_mut() {
        if !v.is_nan() {
            // clamp normalized z to 0..1 for mapping, then map to world z
            let nz = v.clamp(0.0, 1.0);
            *v = nz * z_span + z_min;
        }
    }

    Ok(hm)
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

