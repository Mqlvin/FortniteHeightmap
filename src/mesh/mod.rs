use serde::Deserialize;
use std::{collections::HashMap, path::{Path, PathBuf}};
use ueformat_to_stl::ueformat::{get_vertices_indices_normals, open_uefile};
use crate::{heightmap::error::GenerationError, math::{flat_vec3, transform_vertices}};

#[derive(Debug, Deserialize, Clone)]
pub struct Vec3 {
    #[serde(alias = "X", alias = "Pitch")]
    pub x: f32,
    #[serde(alias = "Y", alias = "Yaw")]
    pub y: f32,
    #[serde(alias = "Z", alias = "Roll")]
    pub z: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MeshEntry {
    #[serde(rename = "Path")]
    pub path: Option<String>,
    #[serde(rename = "Location")]
    pub location: Vec3,
    #[serde(rename = "Rotation")]
    pub rotation: Vec3,
    #[serde(rename = "Scale")]
    pub scale: Vec3,
}

#[derive(Clone)]
pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
}

// load + transform vertices of all actors in world
pub fn load_all_vertices_faces(meshes: &Vec<MeshEntry>, assets_directory: &str) -> Result<(Vec<[f32; 3]>, Vec<[usize; 3]>), GenerationError> {
    let mut cache: HashMap<PathBuf, MeshData> = HashMap::new();
    let mut vertices_all: Vec<[f32; 3]> = Vec::new();
    let mut faces_all: Vec<[usize; 3]> = Vec::new();
    let mut offset = 0usize;

    for mesh in meshes {
        // path relative to assets folder
        let asset_path = match &mesh.path {
            Some(p) if !p.is_empty() => p,
            None => return Err(GenerationError::MeshPathMalformed("Mesh path was None".to_string())),
            _ => continue // empty mesh path
        };

        let relative = asset_path.trim_start_matches("/").split(".").next().ok_or(GenerationError::MeshPathMalformed("Mesh path didn't contain trailing .".to_string()))?;
        let full_path = Path::new(&assets_directory).join(format!("{}.uemodel", relative));

        if !full_path.exists() {
            return Err(GenerationError::MissingMeshPath(full_path.to_str().unwrap_or("no mesh path found").to_string()));
        }

        let base = if let Some(cached) = cache.get(&full_path) {
            cached
        } else {
            let full_path_str = match full_path.as_path().to_str() {
                Some(s) => s,
                None => return Err(GenerationError::MeshPathMalformed("Couldn't get path from PathBuf".to_string()))
            };
            let mut uemodel = open_uefile(full_path_str).map_err(|e| GenerationError::UEFormatError(e))?;
            let (vertices, indices, _normals) = get_vertices_indices_normals(&mut uemodel).map_err(|e| GenerationError::UEFormatError(e))?;
            cache.insert(full_path.clone(), MeshData { vertices, indices });
            cache.get(&full_path).unwrap()
        };

        let loc = flat_vec3(&mesh.location);
        let rot = flat_vec3(&mesh.rotation);
        let scl = flat_vec3(&mesh.scale);

        let transformed = transform_vertices(&base.vertices,
            [loc[0], -loc[1], loc[2]],
            scl,
            [rot[0], rot[2], -rot[1]],
        );

        for v in transformed {
            vertices_all.push(v);
        }
        for f in &base.indices {
            faces_all.push([f[0] as usize + offset, f[1] as usize + offset, f[2] as usize + offset]);
        }
        offset += base.vertices.len();
    }

    if vertices_all.is_empty() || faces_all.is_empty() {
        return Err(GenerationError::LoadMeshError);
    }

    Ok((vertices_all, faces_all))
}


