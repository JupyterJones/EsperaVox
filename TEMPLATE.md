# EsperaVox // LinuxVox Core
> **A Native, Lightweight Voxel Art Studio & Engine for Linux**
> **Created by:** Jack & Esperanza

---

## 1. Project Mission

**EsperaVox** is an open-source, native Linux alternative to MagicaVoxel. Designed for extreme responsiveness and minimal resource overhead, EsperaVox brings intuitive 3D voxel sculpting and path-traced rendering to Linux without requiring compatibility layers like Wine or Proton.

It is split cleanly into two architectures:
1. **LinuxVox Core (`linuxvox-core`)**: A headless, high-performance voxel engine, mesher, raycaster, and `.vox` file I/O library.
2. **EsperaVox (`esperavox`)**: The graphical desktop studio featuring an ergonomic dark-mode UI, real-time 3D viewport, palette management, and responsive brush tooling.

---

## 2. System Architecture

```
+--------------------------------------------------------------------+
|                             EsperaVox                              |
|                (The Artist Studio & User Interface)                |
|  - Viewport & Camera Controller (Orbit / Pan / Zoom / Focus)       |
|  - Tool Palette (Voxel [V], Face [F], Box [B], Line [L], Erase)    |
|  - 256-Color Palette Dock & Color Remapping                        |
|  - Scene Outliner & Transform Controls                             |
|  - Progressive Render Preview                                      |
+--------------------------------------------------------------------+
                                  |
                                  v
+--------------------------------------------------------------------+
|                          LinuxVox Core                             |
|                  (Headless Voxel Engine Library)                   |
|  - VoxelGrid<N>: Dense 3D array storage (8-bit palette indices)    |
|  - Raycasting: 3D DDA (Amanatides-Woo algorithm) for voxel picks   |
|  - Mesher: Multithreaded Greedy Mesher with face culling & AO      |
|  - File I/O: RIFF `.vox` (MagicaVoxel format) parser & serializer  |
|  - Exporters: Wavefront `.obj` (optimized quads), glTF 2.0         |
|  - Undo/Redo Engine: Compressed spatial delta history              |
+--------------------------------------------------------------------+
```

---

## 3. Technology Stack

* **Language:** Rust (2021/2024 edition) — providing memory safety, zero-cost abstractions, and fearless multithreading.
* **UI Framework:** `egui` / `eframe` — immediate-mode GUI with native Wayland/X11 support, instant startup, and tiny memory footprint (~30 MB).
* **3D Math:** `glam` — SIMD-accelerated linear algebra (matrices, vectors, quaternions).
* **Target Hardware:** Optimized for desktop Linux (including AMD Ryzen APUs with Vega integrated graphics and discrete GPUs).

---

## 4. Development Roadmap & Milestones

### Phase 1: Foundation (Milestone 1 — MVP)
- [x] Project architecture design and blueprint.
- [x] Initialize Cargo workspace (`linuxvox-core` and `esperavox`).
- [x] Implement `VoxelGrid` (3D dense volume storage, coordinate math).
- [x] Implement MagicaVoxel default 256-color palette.
- [x] Implement 3D DDA Raycaster for pixel-perfect voxel selection and placement normals.
- [x] Build Face-Culling / Greedy Mesher to convert voxel grids into renderable 3D meshes.
- [x] Implement MagicaVoxel `.vox` file format parser and writer (`MAIN`, `SIZE`, `XYZI`, `RGBA`).
- [x] Build desktop UI with 3D Orbit Camera, wireframe grid floor, and interactive Voxel Add / Erase / Paint.

### Phase 2: Tooling Suite & Palette
- [ ] Implement core editing modes:
  - **Voxel (V):** Single block place, delete, paint.
  - **Face / Extrude (F):** Connected planar extrusion.
  - **Box (B):** 3D rectangular prism fill and hollow.
  - **Line (L):** 3D Bresenham line placement.
- [ ] Symmetry mirrors ($X, Y, Z$).
- [x] Palette editor: MagicaVoxel 256-color saturated spectrum with swatches, hex readout, and color pickers.
- [x] Undo / Redo system with spatial diff history.
- [x] Background styling & 2D image plane backdrop support (solid colors, transparent checkerboard, real PNG/JPG files).
- [x] Interactive Cinematic Timeline Dock (real-time keyframe capture, Catmull-Rom spline playback, and direct MP4 export).

### Phase 3: File Interoperability & Exporters
- [ ] Full `.vox` scene-graph chunk support (`nTRN`, `nGRP`, `nSHP`, `MATL`).
- [ ] Export to `.obj` with texture atlas and vertex colors.
- [ ] Export to `.gltf` / `.glb` with standard PBR material hooks.
- [ ] Slicing and isometric sprite-sheet rendering.

### Phase 4: Path-Traced Render Mode
- [ ] Compute-based progressive raymarcher.
- [ ] Material properties per palette index (Diffuse, Metal, Glass, Emissive).
- [ ] Ground-truth soft shadows, bloom, depth-of-field, and ambient occlusion.

---

## 5. File Structure

```
EsperaVox/
├── Cargo.toml                    # Root workspace configuration
├── TEMPLATE.md                   # Master plan & progress tracking
├── crates/
│   ├── linuxvox-core/            # The headless engine library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── voxel.rs          # Voxel and coordinate types
│   │       ├── grid.rs           # 3D grid storage and manipulation
│   │       ├── palette.rs        # 256-color palette and defaults
│   │       ├── raycast.rs        # 3D DDA raycasting
│   │       ├── mesher.rs         # Greedy meshing and quad generation
│   │       ├── vox_io.rs         # Native .vox binary parser/serializer
│   │       └── history.rs        # Undo/redo delta records
│   └── esperavox/                # The desktop application
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # App entry point
│           ├── app.rs            # Application state & egui panels
│           ├── camera.rs         # 3D Orbit/Pan/Zoom camera
│           ├── viewport.rs       # 3D rendering canvas & ray pick
│           └── ui/               # Modular UI docks
│               ├── palette_dock.rs
│               ├── tool_dock.rs
│               └── menu_bar.rs
```

---

## 6. Verification & Quality Standards

1. **Lightweight Execution:** Zero runtime dependencies beyond standard Linux libraries (`glibc`, graphics driver).
2. **MagicaVoxel Compatibility:** Any standard `.vox` file created in MagicaVoxel must open with exact palette and dimensions in EsperaVox.
3. **Smooth Interaction:** Viewport updates, mouse raycasting, and meshing must run at >= 60 FPS on integrated APU graphics.
