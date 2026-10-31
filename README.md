# Rustcraft [Vulkan]

An ultra-fast, multithreaded Minecraft recreation written from scratch in Rust, utilizing Vulkan via `wgpu`.

## Features

1. **Title Screen & Menus**:
   * **Starting Menu**: Play World in Survival mode or Creative mode, configure Graphics Settings, or Quit.
   * **Pause Menu (ESC)**: Seamlessly resume game, switch game mode on the fly between Survival and Creative, tweak settings, or return to title.
   * **Game Over Screen**: Red vignette overlay with Respawn and Return to Title options.

2. **Creative & Survival Game Modes**:
   * **Creative Mode**: Free flight mode (`F` to toggle, `Space` to ascend, `Shift` to descend), instant block placing and breaking, invulnerability.
   * **Survival Mode**: 10 Heart (20 HP) health system, swept AABB physics with gravity and friction, realistic fall damage when falling from $>3.5$ blocks, and natural regeneration over time.

3. **Inventory & Palette System ('E')**:
   * Press `E` at any time to open the **Creative Inventory & Palette**.
   * Grid of available blocks (Grass, Dirt, Stone, Cobblestone, Wood, Leaves, Sand, Planks, Glass, Bedrock).
   * Click any block in the palette to assign it to your active hotbar slot.
   * Click any hotbar slot (1–9) to change selection.
   * Press `E` or `ESC` to return to the game.

4. **Graphics Settings**:
   * **Render Distance**: Dynamic cycling between 4, 8, 12, and 16 chunks (up to 33x33 = 1,089 chunks).
   * **Field of View (FOV)**: 60°, 75°, 90°, and 105°.
   * **Atmospheric Fog**: Toggleable smooth distance fog blending into the sky clear color.
   * **Vertical Sync (VSync)**: Toggleable between uncapped peak FPS (`AutoNoVsync`) and synchronized refresh (`AutoVsync`).

5. **Graphics & Performance**:
   * Direct **Vulkan** backend on AMD Radeon 890M Graphics.
   * 2D Texture Array eliminating texture atlas seams and mip bleeding.
   * 4-level Smooth Ambient Occlusion with anisotropic quad flipping.
   * Multithreaded chunk terrain generation and meshing with **Rayon**.

---

## Controls

| Key / Mouse | Action |
|---|---|
| **W, A, S, D** | Move Forward / Left / Backward / Right |
| **Space** | Jump (or Fly Up when Flying in Creative) |
| **Left Shift** | Sneak (or Fly Down in Creative) |
| **Left Ctrl** | Sprint |
| **F** | Toggle Flying Mode (Creative Mode only) |
| **1 - 9** | Hotbar slot selection |
| **E** | Open / Close Inventory & Block Palette |
| **Escape** | Pause Menu / Close open menu / Unlock Cursor |
| **Left Click** | Break block (or Click UI buttons/slots) |
| **Right Click** | Place active block |

---

## Running

```bash
cargo run --release
```
