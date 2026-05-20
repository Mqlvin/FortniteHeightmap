use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use ueformat_to_stl::ueformat::{get_vertices_indices_normals, open_uefile};
use crate::{chunk::{get_terrain_chunk, identify_chunks, identify_meshes}, heightmap::{error::GenerationError, export::{ExportData, write_export}, image::{rasterize_heightmap, save_heightmap_png}, math::{flat_vec3, transform_vertices}, mesh::{MeshData, MeshEntry}}};

mod math;
mod image;
mod export;
pub mod mesh;
pub mod error;

pub fn generate_heightmap(chunk_directory: &str, assets_directory: &str, save_directory: &str, output_size: u32, save_terrain_separately: bool) -> Result<(), GenerationError> {
    fs::create_dir_all(save_directory).map_err(|e| GenerationError::FileIO(e))?;

    let chunk_files: Vec<PathBuf> = identify_chunks(chunk_directory)?;
    let terrain_chunk = get_terrain_chunk(&chunk_files);
    if save_terrain_separately && terrain_chunk.is_none() { // do this up here to not waste users time if it is None
        return Err(GenerationError::ChunkError("Couldn't identify terrains chunk".to_string().into()));
    }
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

    let (result, scale_used) = rasterize_heightmap(
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

    println!("Saving map height data as image");
    let save_path = format!("{}/heightmap.png", save_directory);
    save_heightmap_png(&save_path, &result, output_size, output_size, min_z as f32, max_z as f32)?;

    drop(vertices_all); drop(faces_all); // drop memory here in before loading just terrain mesh data

    if save_terrain_separately {
        // unfortunately this has to be here. we must load all vertices of all meshes to identify
        // bounds or else map dimensions will potentially be mismatched. further, by using the same
        // min_z and max_z as the real map the two heightmaps will have identical greys for
        // identical heights.

        println!("Saving separate terrain heightmap");
        
        let terrain_meshes = identify_meshes(&vec![terrain_chunk.expect("Already unwrapped").clone()])?;
        let (vertices_terrain, faces_terrain) = load_all_vertices_faces(&terrain_meshes, assets_directory)?;
        let (terrain_result, _) = rasterize_heightmap(
            &vertices_terrain,
            &faces_terrain,
            min_x,
            max_x,
            min_y,
            max_y,
            min_z.floor() - 1.,
            max_z.floor() + 1.,
            output_size,
            output_size
        )?;

        println!("Saving terrain height data as image");
        let save_path = format!("{}/terrainmap.png", save_directory);
        save_heightmap_png(&save_path, &terrain_result, output_size, output_size, min_z as f32, max_z as f32)?;
    }


    // getting height of a single wall here
    let height_span = max_z.ceil() + 1. - (min_z.floor() - 1.);
    let metre_16bit = (6553500. / height_span).round() as u16;

    let export_data = ExportData {
        min_x,
        max_x,
        min_y,
        max_y,
        min_z,
        max_z,
        metre_16bit,
        metre_px: scale_used * 100.,
        total_meshes: meshes.len()
    };

    if let Err(err) = write_export(&format!("{}/mapdata.json", save_directory), &export_data) {
        return Err(err);
    }

    Ok(())
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
