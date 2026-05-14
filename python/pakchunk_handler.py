import json
import os
import helpers

def handle_pakchunk(chunk: json, print_dirs: bool):
    if not chunk["MetaData"]:
        print(f"Critical error parsing chunk: no MetaData ({json.dumps(chunk)[0:200]})")

    exports, name = split_pakchunk(chunk)
    if len(exports) == 0:
        print("No exports in PAKCHUNK, skipping")
        return

    pre_cull = len(exports)
    exports = cull_exports(exports)
    print(f"Culled: {pre_cull - len(exports)} ({round((pre_cull - len(exports)) / pre_cull, 1)}) objects from {name}")

    chunk["Exports"][0]["Meshes"] = exports
    helpers.write_pakchunk(chunk, name)

    if print_dirs:
        asset_root = chunk["MetaData"]["AssetsRoot"]
        print("When using the heightmap generator, you will need to input the following folders:")
        print(f"Assets path: '{os.path.abspath(asset_root)}'")
        print(f"Chunks path: '{os.path.abspath(helpers.get_save_directory(''))}'")


def split_pakchunk(chunk: json):
    if not chunk["Exports"]:
        print(f"Critical error parsing chunk: no Exports ({json.dumps(chunk)[0:200]})")
        return
    exports = chunk["Exports"][0]

    if not exports["Name"]:
        print(f"Critical error parsing chunk: no Exports.Name ({json.dumps(chunk)[0:200]})")
        return
    name = exports["Name"]

    return exports["Meshes"], name


def cull_exports(exports):
    exports = list(filter(lambda e: not cull(e), exports))
    return exports

def cull(mesh_obj: any):
    mesh_path = mesh_obj["Path"]
    mesh_name = helpers.get_asset_name_from_path(mesh_path)
    for keyword in ["bush", "plant", "wavygrass", "shrub", "flowers"]:
        if keyword.lower() in mesh_name.lower():
            return True

    path_blacklists = [
        "/Engine/BasicShapes/Plane.Plane",
        "/Environments/Landscape/Meshes/Background",
    ]
    for path_blacklist in path_blacklists:
        if path_blacklist.lower() in mesh_path.lower():
            return True

    return False