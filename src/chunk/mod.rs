use std::{fs, path::PathBuf};
use serde::Deserialize;
use crate::{heightmap::error::GenerationError, mesh::MeshEntry};


#[derive(Debug, Deserialize)]
pub struct ChunkFile {
    #[serde(rename = "Exports")]
    pub exports: Vec<Export>,
}

#[derive(Debug, Deserialize)]
pub struct Export {
    #[serde(rename = "Meshes")]
    pub meshes: Vec<MeshEntry>,
}


// identify chunks of exports to load
pub fn identify_chunks(dir: &str) -> Result<Vec<PathBuf>, GenerationError> {
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
pub fn identify_meshes(chunks: &Vec<PathBuf>) -> Result<Vec<MeshEntry>, GenerationError> {
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

pub fn get_terrain_chunk(chunks: &Vec<PathBuf>) -> Option<&PathBuf> {
    chunks
        .iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_str()?;
            Some((p, name))
        })
        .find(|(_, name)| name.len() != 25 && !name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()))
        .map(|(path, _)| path)
}
