use std::{collections::HashMap, fs, path::{Path, PathBuf}};

use ueformat_to_stl::ueformat::{get_vertices_indices_normals, open_uefile};

use crate::heightmap::{error::GenerationError, export::{ExportData, write_export}, image::{rasterize_heightmap, save_heightmap_png}, math::{flat_vec3, transform_vertices}, mesh::{ChunkFile, MeshData, MeshEntry}};

mod math;
mod image;
mod mesh;
mod export;
pub mod error;

pub fn generate_heightmap(chunk_directory: &str, assets_directory: &str, save_path: &str, output_size: u32) -> Result<(), GenerationError> {
    let chunk_files: Vec<PathBuf> = identify_chunks(chunk_directory)?;
    println!("Identified {} PAKCHUNKS", chunk_files.len());

    let meshes: Vec<MeshEntry> = identify_meshes(&chunk_files)?;
    println!("Identified {} meshes", meshes.len());

    let (vertices_all, faces_all) = load_all_vertices_faces(&meshes, assets_directory)?;

    println!("Loaded meshes, getting minmax");

    let min_x = vertices_all.iter().map(|v| v[0]).fold(f32::INFINITY, f32::min);
    let max_x = vertices_all.iter().map(|v| v[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_y = vertices_all.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min);
    let max_y = vertices_all.iter().map(|v| v[1]).fold(f32::NEG_INFINITY, f32::max);
    let min_z = vertices_all.iter().map(|v| v[2]).fold(f32::INFINITY, f32::min);
    let max_z = vertices_all.iter().map(|v| v[2]).fold(f32::NEG_INFINITY, f32::max);

    println!("Got minmax: {} {} {} {} {} {}", min_x, max_x, min_y, max_y, min_z, max_z);

    let result = rasterize_heightmap(
        &vertices_all,
        &faces_all,
        min_x,
        max_x,
        min_y,
        max_y,
        min_z.floor() - 1., // give a bit of room
        max_z.ceil() + 1.,
        output_size,
        output_size,
    )?;

    println!("Saving height data as image");
    save_heightmap_png(save_path, &result, output_size, output_size)?;

    // getting height of a single wall here
    let height_span = max_z.ceil() + 1. - (min_z.floor() - 1.);
    let tile_height_16bit = ((384. / height_span) * 65535.).round() as u16;

    let export_data = ExportData {
        min_x,
        max_x,
        min_y,
        max_y,
        min_z,
        max_z,
        tile_height_16bit,
        total_meshes: meshes.len()
    };

    if let Err(err) = write_export(&format!("{}.json", save_path), &export_data) {
        return Err(err);
    }

    Ok(())
}


// identify chunks of exports to load
fn identify_chunks(dir: &str) -> Result<Vec<PathBuf>, GenerationError> {
    Ok(fs::read_dir(dir)
        .map_err(GenerationError::FileIO)?
        .map(|entry| entry.map_err(GenerationError::FileIO))
        .map(|entry| entry.map(|e| e.path()))
        .filter_map(|result| match result {
            Ok(path) if path.is_file() => Some(Ok(path)),
            Ok(_) => None,
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<Vec<_>, _>>()?)
}

// identify meshes in scene
fn identify_meshes(chunks: &Vec<PathBuf>) -> Result<Vec<MeshEntry>, GenerationError> {
    let mut meshes: Vec<MeshEntry> = Vec::with_capacity(chunks.len() * 200); // avg about 200 meshes / chunk
    for file in chunks {
        let text = fs::read_to_string(file).map_err(|e| GenerationError::FileIO(e))?;
        let chunk_data: ChunkFile = serde_json::from_str(&text).map_err(|e| GenerationError::ChunkError(Box::new(e)))?;
        if let Some(export) = chunk_data.exports.get(0) {
            meshes.extend(export.meshes.clone());           
        }
    }
    Ok(meshes)
}

// load + transform vertices of all actors in world
fn load_all_vertices_faces(meshes: &Vec<MeshEntry>, assets_directory: &str) -> Result<(Vec<[f32; 3]>, Vec<[usize; 3]>), GenerationError> {
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
