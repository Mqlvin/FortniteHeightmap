use image::ImageError;
use thiserror::Error;
use ueformat_to_stl::ueformat;

#[derive(Error, Debug)]
pub enum GenerationError {
    #[error("Error during IO operations: {0}")]
    FileIO(std::io::Error),

    #[error("Error during chunk handling: {0}")]
    ChunkError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Missing mesh path: {0}")]
    MissingMeshPath(String),

    #[error("Mesh path was malformed: {0}")]
    MeshPathMalformed(String),

    #[error("UEFormat error: {0}")]
    UEFormatError(ueformat::error::ParseError),

    #[error("Error loading meshes: no vertices or faces")]
    LoadMeshError,

    #[error("Error rasterizing heightmap: {0}")]
    MapRasterizationError(String),

    #[error("Error saving heightmap image: {0}")]
    ImageWriteError(ImageError),
}
