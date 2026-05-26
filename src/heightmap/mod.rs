use std::{fs, path::PathBuf};
use crate::{chunk::{get_terrain_chunk, identify_chunks, identify_meshes}, heightmap::{error::GenerationError, io::{ExportData, save_heightmap_png, write_export}, rasterize::rasterize_heightmap}, mesh::{MeshEntry, load_all_vertices_faces}};

mod rasterize;
mod io;
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


