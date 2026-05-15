# Usage

**Step 1: Install Prerequisites**<br>
Install prerequisites, including Fortnite, FortnitePorting and Blender. Ensure you have played a match on the map(s) you would like to export.

<br>

**Step 2: Add Blender plugin to FortnitePorting**<br>
Open FortnitePorting. In the left hand side, click `Plugin`, and in the top right `+ Add Version`. You will need to point this torwards your Blender installations primary `.exe` file.
Once you can see the Blender installation in FortnitePorting, close any instances of Blender you have open.

<br>

**Step 3: Configure FortnitePorting settings**<br>
Open the `Export Settings` tab, and select `Blender`. Configure the following settings:<br>
In `General`:
- Scale down: ON
- Compression: ZStandard Compression

In `Mesh`:
- Level of Detail: 2
- Export Nanite: OFF
- Polygon Type: Triangles
- Import Collision Geometry: OFF

In `Material`:
- Export: OFF

<br>

**Step 4: Run `export_server.py`**<br>
Run `python/export_server.py` with Python. Alternatively, if you do not have Python installed, there may be a binary release on the Releases tab in GitHub.

<br>

**Step 5: Export Fortnite assets**<br>
In FortnitePorting, press `Map`.<br>
At the top right of the screen, press `Select All`, and at the bottom of the screen, press `Include Main Level` (in this order). In Flags, ensure **only** `Landscape`, `Actors` and `Instanced Actors` are enabled.<br>
Press `Export to Blender` and wait for this to complete.
> Once complete, close FortnitePorting, but leave `export_server.py` running as it has printed some valuable information for the next steps.

<br>

**Step 6: Open the Heightmap Generator**<br>
Open the Fortnite Heightmap generator executable downloaded from this repository. You will need to fill out two boxes in the *Input* section:
- Assets folder - Select the folder containing Fortnite assets
- Chunks folder - Select the folder containing chunks
> Both folders' locations were printed in the `export_server.py` console window, you can just copy these across

<br>

**Step 7: Generate the Heightmap**<br>
Now you may adjust the output resolution, and select whether you would like a terrain-only map to be additionally exported.
Finally, press "Generate" and wait for the result - the window will hang / freeze temporarily.
The resulting files will be located in `out/...`
