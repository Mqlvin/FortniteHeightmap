use rayon::prelude::*;
use std::sync::Mutex;

use crate::{heightmap::{error::GenerationError, io::AdvancedSettings}, math::edge};

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
    height: u32,
    settings: &AdvancedSettings,
) -> Result<(Vec<f32>, u32, u32, f32), GenerationError> {
    let x_span = x_max - x_min;
    let y_span = y_max - y_min;
    let z_span = z_max - z_min;

    if x_span == 0.0 || y_span == 0.0 || z_span == 0.0 {
        return Err(GenerationError::MapRasterizationError(
            "Bounds must have non-zero extent".to_string(),
        ));
    }

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

    let crop_x_min = x_min + x_span * trim_left;
    let crop_x_max = x_max - x_span * trim_right;

    let crop_y_top = y_min + y_span * trim_top;
    let crop_y_bottom = y_max - y_span * trim_bottom;

    let crop_x_span = crop_x_max - crop_x_min;
    let crop_y_span = crop_y_bottom - crop_y_top;

    if crop_x_span == 0.0 || crop_y_span == 0.0 {
        return Err(GenerationError::MapRasterizationError(
            "Crop bounds collapsed to zero extent".to_string(),
        ));
    }

    let scale_x = (crop_w as f32 - 1.0) / crop_x_span.abs();
    let scale_y = (crop_h as f32 - 1.0) / crop_y_span.abs();
    let scale = scale_x.min(scale_y);

    let out_w = crop_x_span.abs() * scale;
    let out_h = crop_y_span.abs() * scale;

    let pad_x = (crop_w as f32 - out_w) * 0.5;
    let pad_y = (crop_h as f32 - out_h) * 0.5;

    let hm = Mutex::new(vec![f32::NAN; (crop_w * crop_h) as usize]);
    let eps = 1e-7f32;

    faces.par_iter().for_each(|face| {
        let v0 = vertices[face[0]];
        let v1 = vertices[face[1]];
        let v2 = vertices[face[2]];

        let to_screen = |v: [f32; 3]| -> [f32; 3] {
            let sx = (v[0] - crop_x_min) * scale + pad_x;
            let sy = (crop_y_bottom - v[1]) * scale + pad_y;
            let rz = v[2];
            [sx, sy, rz]
        };

        let p0 = to_screen(v0);
        let p1 = to_screen(v1);
        let p2 = to_screen(v2);

        let min_x = p0[0].min(p1[0]).min(p2[0]).floor().max(0.0) as i32;
        let max_x = p0[0].max(p1[0]).max(p2[0]).ceil().min((crop_w as f32) - 1.0) as i32;
        let min_y = p0[1].min(p1[1]).min(p2[1]).floor().max(0.0) as i32;
        let max_y = p0[1].max(p1[1]).max(p2[1]).ceil().min((crop_h as f32) - 1.0) as i32;

        if min_x > max_x || min_y > max_y {
            return;
        }

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
                ax < bx
            } else {
                ay < by
            }
        };

        let mut local_updates: Vec<(usize, f32)> = Vec::new();

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                let w0 = edge([b[0], b[1]], [c[0], c[1]], [px, py]);
                let w1 = edge([c[0], c[1]], [a[0], a[1]], [px, py]);
                let w2 = edge([a[0], a[1]], [b[0], b[1]], [px, py]);

                if ccw {
                    if w0 < -eps || w1 < -eps || w2 < -eps {
                        continue;
                    }
                } else if w0 > eps || w1 > eps || w2 > eps {
                    continue;
                }

                if w0.abs() <= eps && !is_top_left(b[0], b[1], c[0], c[1]) {
                    continue;
                }
                if w1.abs() <= eps && !is_top_left(c[0], c[1], a[0], a[1]) {
                    continue;
                }
                if w2.abs() <= eps && !is_top_left(a[0], a[1], b[0], b[1]) {
                    continue;
                }

                let l0 = w0 / area;
                let l1 = w1 / area;
                let l2 = w2 / area;

                let z = l0 * p0[2] + l1 * p1[2] + l2 * p2[2];
                let idx = (y as u32 * crop_w + x as u32) as usize;
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

    Ok((hm.into_inner().unwrap(), crop_w, crop_h, scale))
}
