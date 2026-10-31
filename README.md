# High-Performance Voxel Engine in Rust (Vulkan)

An ultra-fast, multithreaded Minecraft-style voxel game engine written from scratch in Rust, utilizing Vulkan via `wgpu`.

## Key Performance Features

1. **Direct Vulkan Pipeline**:
   - Vulkan backend (`wgpu::Backends::VULKAN`) with asynchronous compute and transfer capabilities.
   - Dedicated 2D Texture Array (`texture_2d_array`) with nearest-neighbor sampling eliminating mip-map bleeding and texture atlas seams.
   - Depth-tested backface culling (`Depth32Float` + `Face::Back`).

2. **Multithreaded World Generation & Meshing**:
   - Terrain generation and 3D cave sculpting running in parallel across all CPU cores via **Rayon**.
   - Asynchronous chunk meshing with automatic boundary-aware culling.

3. **High-Efficiency Mesh Architecture**:
   - Face visibility culling: Hidden internal faces between opaque voxels are discarded immediately.
   - 4-level Smooth Ambient Occlusion (AO) computed per vertex with anisotropic diagonal quad flipping to avoid lighting seams.
   - Directional sunlight and ambient sky lighting combined in the WGSL shader.
   - Frustum and distance culling for distant chunks.

4. **Player Physics & Voxel Raycasting**:
   - Continuous swept AABB collision resolution against voxel blocks.
   - Amanatides-Woo Fast Voxel Traversal algorithm (DDA) for raycasting up to 7 blocks for block breaking and placing.

---

## Controls

| Key / Action | Action |
|---|---|
| **W, A, S, D** | Move Forward / Left / Backward / Right |
| **Space** | Jump (or Fly Up when Flying is enabled) |
| **Left Shift** | Sneak (or Fly Down when Flying is enabled) |
| **Left Ctrl** | Sprint |
| **F** | Toggle Flying Mode (Default: ON at spawn) |
| **1 - 9** | Select block to place (Grass, Dirt, Stone, Cobble, Wood, Leaves, Sand, Planks, Glass) |
| **Left Mouse Button** | Break block |
| **Right Mouse Button** | Place selected block |
| **Escape** | Toggle mouse cursor lock |

---

## How to Run

```bash
# Run in development mode
cargo run

# Run in optimized release mode (maximum FPS)
cargo run --release
```
