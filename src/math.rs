use std::f32::consts::PI;
use crate::mesh::Vec3;


pub fn flat_vec3(v: &Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

pub fn euler_matrix_sxyz(x: f32, y: f32, z: f32) -> [[f32; 3]; 3] {
    let (sx, cx) = x.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sz, cz) = z.sin_cos();

    [
        [cy * cz, -cy * sz, sy],
        [cx * sz + cz * sx * sy, cx * cz - sx * sy * sz, -cy * sx],
        [sx * sz - cx * cz * sy, cz * sx + cx * sy * sz, cx * cy],
    ]
}

pub fn transform_vertices(verts: &[[f32; 3]], translate: [f32; 3], scale: [f32; 3], euler_deg: [f32; 3]) -> Vec<[f32; 3]> {
    let rx = euler_deg[0] * PI / 180.0;
    let ry = euler_deg[1] * PI / 180.0;
    let rz = euler_deg[2] * PI / 180.0;
    let r = euler_matrix_sxyz(rx, ry, rz);

    verts.iter().map(|v| {
        let mut p = [v[0] * scale[0], v[1] * scale[1], v[2] * scale[2]];
        let x = p[0] * r[0][0] + p[1] * r[0][1] + p[2] * r[0][2];
        let y = p[0] * r[1][0] + p[1] * r[1][1] + p[2] * r[1][2];
        let z = p[0] * r[2][0] + p[1] * r[2][1] + p[2] * r[2][2];
        p[0] = x + translate[0];
        p[1] = y + translate[1];
        p[2] = z + translate[2];
        p
    }).collect()
}

pub fn edge(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    (p[0] - a[0]) * (b[1] - a[1]) - (p[1] - a[1]) * (b[0] - a[0])
}

