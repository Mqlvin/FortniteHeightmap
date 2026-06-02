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
) -> Result<(Vec<f32>, Vec<usize>, u32, u32, f32), GenerationError> {
    let x_span = x_max - x_min;
    let y_span = y_max - y_min;
    let z_span = z_max - z_min;

    if x_span == 0.0 || y_span == 0.0 || z_span == 0.0 {
        return Err(GenerationError::MapRasterizationError(
            "Bounds must have non-zero extent".to_string(),
        ));
    }

    let clamp_pct = |v: f32| v.clamp(0.0, 100.0) * 0.01;

    let trim_top = clamp_pct(settings.trim_top);
    let trim_right = clamp_pct(settings.trim_right);
    let trim_bottom = clamp_pct(settings.trim_bottom);
    let trim_left = clamp_pct(settings.trim_left);

    let crop_x_min = x_min + x_span * trim_left;
    let crop_x_max = x_max - x_span * trim_right;
    let crop_y_min = y_min + y_span * trim_top;
    let crop_y_max = y_max - y_span * trim_bottom;

    let crop_x_span = crop_x_max - crop_x_min;
    let crop_y_span = crop_y_max - crop_y_min;

    if crop_x_span <= 0.0 || crop_y_span <= 0.0 {
        return Err(GenerationError::MapRasterizationError(
            "Crop bounds collapsed to zero extent".to_string(),
        ));
    }

    let base_w = width.max(1);
    let base_h = height.max(1);

    let scale_w = (base_w.saturating_sub(1).max(1)) as f32 / crop_x_span;
    let scale_h = (base_h.saturating_sub(1).max(1)) as f32 / crop_y_span;
    let scale = scale_w.min(scale_h);

    let out_w = if scale == scale_w {
        base_w
    } else {
        (crop_x_span * scale + 1.0).round().max(1.0) as u32
    };

    let out_h = if scale == scale_h {
        base_h
    } else {
        (crop_y_span * scale + 1.0).round().max(1.0) as u32
    };

    let hm = Mutex::new(vec![f32::NAN; (out_w * out_h) as usize]);
    let face_map = Mutex::new(vec![usize::MAX; (out_w * out_h) as usize]);
    let eps = 1e-7f32;
    let inv_area_eps = 1e-12f32;

    faces.par_iter().enumerate().for_each(|(face_id, face)| {
        let [i0, i1, i2] = *face;
        let v0 = vertices[i0];
        let v1 = vertices[i1];
        let v2 = vertices[i2];

        let to_screen = |v: [f32; 3]| -> [f32; 3] {
            [
                (v[0] - crop_x_min) * scale,
                (crop_y_max - v[1]) * scale,
                v[2],
            ]
        };

        let p0 = to_screen(v0);
        let p1 = to_screen(v1);
        let p2 = to_screen(v2);

        let min_x = p0[0].min(p1[0]).min(p2[0]).floor().max(0.0) as i32;
        let max_x = p0[0].max(p1[0]).max(p2[0]).ceil().min(out_w as f32 - 1.0) as i32;
        let min_y = p0[1].min(p1[1]).min(p2[1]).floor().max(0.0) as i32;
        let max_y = p0[1].max(p1[1]).max(p2[1]).ceil().min(out_h as f32 - 1.0) as i32;

        if min_x > max_x || min_y > max_y {
            return;
        }

        let a = [p0[0], p0[1]];
        let b = [p1[0], p1[1]];
        let c = [p2[0], p2[1]];
        let area = edge(a, b, c);

        if area.abs() < inv_area_eps {
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

        let mut updates = Vec::new();

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

                let z = (w0 * p0[2] + w1 * p1[2] + w2 * p2[2]) / area;
                let idx = (y as u32 * out_w + x as u32) as usize;
                updates.push((idx, z, face_id));
            }
        }

        let mut hm_guard = hm.lock().unwrap();
        let mut face_map_guard = face_map.lock().unwrap();
        for (idx, z, face_id) in updates {
            let current = &mut hm_guard[idx];
            if current.is_nan() || z > *current {
                *current = z;
                face_map_guard[idx] = face_id;
            }

        }
    });

    Ok((hm.into_inner().unwrap(), face_map.into_inner().unwrap(), out_w, out_h, scale))
}
