use serde::Deserialize;

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
