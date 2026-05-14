import os
import json

def get_asset_name_from_path(mesh_path: str):
    return mesh_path.split("/")[-1]

def create_export_folder():
    if not os.path.exists("chunks"):
        os.makedirs("chunks")

def get_save_directory(chunk_name: str):
    return f"chunks/{chunk_name}"

def write_pakchunk(chunk: any, name: str):
    with open(get_save_directory(name), "w") as f:
        f.write(json.dumps(chunk, indent=4))