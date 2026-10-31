use super::vertex::BlockVertex;
use crate::voxel::block::BlockType;
use crate::voxel::chunk::{Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::voxel::world::World;

pub struct ChunkMesh {
    pub vertices: Vec<BlockVertex>,
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

pub struct Mesher;

impl Mesher {
    pub fn mesh_chunk(chunk: &Chunk, world: &World) -> ChunkMesh {
        let mut vertices = Vec::with_capacity(4096);
        let mut indices = Vec::with_capacity(6144);

        let origin_x = chunk.pos.x * CHUNK_WIDTH as i32;
        let origin_z = chunk.pos.z * CHUNK_DEPTH as i32;

        for ly in 0..CHUNK_HEIGHT {
            let gy = ly as i32;
            for lz in 0..CHUNK_DEPTH {
                let gz = origin_z + lz as i32;
                for lx in 0..CHUNK_WIDTH {
                    let gx = origin_x + lx as i32;

                    let block = chunk.get_block(lx, ly, lz);
                    if block == BlockType::Air {
                        continue;
                    }

                    let fx = gx as f32;
                    let fy = gy as f32;
                    let fz = gz as f32;

                    // Special slim 3D post mesh for Torch!
                    if block == BlockType::Torch {
                        Self::push_torch_mesh(&mut vertices, &mut indices, fx, fy, fz);
                        continue;
                    }

                    let emit_boost = Self::local_light_boost(world, gx, gy, gz, block);
                    let top_y = if block == BlockType::Water && world.get_block(gx, gy + 1, gz) == BlockType::Air {
                        fy + 0.88
                    } else {
                        fy + 1.0
                    };

                    // Face 0: +X (Right / East, normal = (+1, 0, 0))
                    if Self::is_face_visible(world.get_block(gx + 1, gy, gz), block) {
                        let tex_layer = block.texture_layer(0);
                        let ao = [
                            (Self::vertex_ao(world, gx + 1, gy - 1, gz, gx + 1, gy, gz + 1, gx + 1, gy - 1, gz + 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx + 1, gy - 1, gz, gx + 1, gy, gz - 1, gx + 1, gy - 1, gz - 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx + 1, gy + 1, gz, gx + 1, gy, gz - 1, gx + 1, gy + 1, gz - 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx + 1, gy + 1, gz, gx + 1, gy, gz + 1, gx + 1, gy + 1, gz + 1) + emit_boost).min(1.6),
                        ];
                        let quad = [
                            BlockVertex { position: [fx + 1.0, fy, fz + 1.0], normal_idx: 0, uv: [0.0, 1.0], tex_layer, ao: ao[0] },
                            BlockVertex { position: [fx + 1.0, fy, fz], normal_idx: 0, uv: [1.0, 1.0], tex_layer, ao: ao[1] },
                            BlockVertex { position: [fx + 1.0, top_y, fz], normal_idx: 0, uv: [1.0, 0.0], tex_layer, ao: ao[2] },
                            BlockVertex { position: [fx + 1.0, top_y, fz + 1.0], normal_idx: 0, uv: [0.0, 0.0], tex_layer, ao: ao[3] },
                        ];
                        Self::push_quad(&mut vertices, &mut indices, quad, ao);
                    }

                    // Face 1: -X (Left / West, normal = (-1, 0, 0))
                    if Self::is_face_visible(world.get_block(gx - 1, gy, gz), block) {
                        let tex_layer = block.texture_layer(1);
                        let ao = [
                            (Self::vertex_ao(world, gx - 1, gy - 1, gz, gx - 1, gy, gz - 1, gx - 1, gy - 1, gz - 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx - 1, gy - 1, gz, gx - 1, gy, gz + 1, gx - 1, gy - 1, gz + 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx - 1, gy + 1, gz, gx - 1, gy, gz + 1, gx - 1, gy + 1, gz + 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx - 1, gy + 1, gz, gx - 1, gy, gz - 1, gx - 1, gy + 1, gz - 1) + emit_boost).min(1.6),
                        ];
                        let quad = [
                            BlockVertex { position: [fx, fy, fz], normal_idx: 1, uv: [0.0, 1.0], tex_layer, ao: ao[0] },
                            BlockVertex { position: [fx, fy, fz + 1.0], normal_idx: 1, uv: [1.0, 1.0], tex_layer, ao: ao[1] },
                            BlockVertex { position: [fx, top_y, fz + 1.0], normal_idx: 1, uv: [1.0, 0.0], tex_layer, ao: ao[2] },
                            BlockVertex { position: [fx, top_y, fz], normal_idx: 1, uv: [0.0, 0.0], tex_layer, ao: ao[3] },
                        ];
                        Self::push_quad(&mut vertices, &mut indices, quad, ao);
                    }

                    // Face 2: +Y (Top, normal = (0, +1, 0))
                    if Self::is_face_visible(world.get_block(gx, gy + 1, gz), block) {
                        let tex_layer = block.texture_layer(2);
                        let ao = [
                            (Self::vertex_ao(world, gx - 1, gy + 1, gz, gx, gy + 1, gz + 1, gx - 1, gy + 1, gz + 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx + 1, gy + 1, gz, gx, gy + 1, gz + 1, gx + 1, gy + 1, gz + 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx + 1, gy + 1, gz, gx, gy + 1, gz - 1, gx + 1, gy + 1, gz - 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx - 1, gy + 1, gz, gx, gy + 1, gz - 1, gx - 1, gy + 1, gz - 1) + emit_boost).min(1.6),
                        ];
                        let quad = [
                            BlockVertex { position: [fx, top_y, fz + 1.0], normal_idx: 2, uv: [0.0, 1.0], tex_layer, ao: ao[0] },
                            BlockVertex { position: [fx + 1.0, top_y, fz + 1.0], normal_idx: 2, uv: [1.0, 1.0], tex_layer, ao: ao[1] },
                            BlockVertex { position: [fx + 1.0, top_y, fz], normal_idx: 2, uv: [1.0, 0.0], tex_layer, ao: ao[2] },
                            BlockVertex { position: [fx, top_y, fz], normal_idx: 2, uv: [0.0, 0.0], tex_layer, ao: ao[3] },
                        ];
                        Self::push_quad(&mut vertices, &mut indices, quad, ao);
                    }

                    // Face 3: -Y (Bottom, normal = (0, -1, 0))
                    if Self::is_face_visible(world.get_block(gx, gy - 1, gz), block) {
                        let tex_layer = block.texture_layer(3);
                        let ao = [
                            (Self::vertex_ao(world, gx - 1, gy - 1, gz, gx, gy - 1, gz - 1, gx - 1, gy - 1, gz - 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx + 1, gy - 1, gz, gx, gy - 1, gz - 1, gx + 1, gy - 1, gz - 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx + 1, gy - 1, gz, gx, gy - 1, gz + 1, gx + 1, gy - 1, gz + 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx - 1, gy - 1, gz, gx, gy - 1, gz + 1, gx - 1, gy - 1, gz + 1) + emit_boost).min(1.6),
                        ];
                        let quad = [
                            BlockVertex { position: [fx, fy, fz], normal_idx: 3, uv: [0.0, 1.0], tex_layer, ao: ao[0] },
                            BlockVertex { position: [fx + 1.0, fy, fz], normal_idx: 3, uv: [1.0, 1.0], tex_layer, ao: ao[1] },
                            BlockVertex { position: [fx + 1.0, fy, fz + 1.0], normal_idx: 3, uv: [1.0, 0.0], tex_layer, ao: ao[2] },
                            BlockVertex { position: [fx, fy, fz + 1.0], normal_idx: 3, uv: [0.0, 0.0], tex_layer, ao: ao[3] },
                        ];
                        Self::push_quad(&mut vertices, &mut indices, quad, ao);
                    }

                    // Face 4: +Z (South, normal = (0, 0, +1))
                    if Self::is_face_visible(world.get_block(gx, gy, gz + 1), block) {
                        let tex_layer = block.texture_layer(4);
                        let ao = [
                            (Self::vertex_ao(world, gx - 1, gy, gz + 1, gx, gy - 1, gz + 1, gx - 1, gy - 1, gz + 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx + 1, gy, gz + 1, gx, gy - 1, gz + 1, gx + 1, gy - 1, gz + 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx + 1, gy, gz + 1, gx, gy + 1, gz + 1, gx + 1, gy + 1, gz + 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx - 1, gy, gz + 1, gx, gy + 1, gz + 1, gx - 1, gy + 1, gz + 1) + emit_boost).min(1.6),
                        ];
                        let quad = [
                            BlockVertex { position: [fx, fy, fz + 1.0], normal_idx: 4, uv: [0.0, 1.0], tex_layer, ao: ao[0] },
                            BlockVertex { position: [fx + 1.0, fy, fz + 1.0], normal_idx: 4, uv: [1.0, 1.0], tex_layer, ao: ao[1] },
                            BlockVertex { position: [fx + 1.0, top_y, fz + 1.0], normal_idx: 4, uv: [1.0, 0.0], tex_layer, ao: ao[2] },
                            BlockVertex { position: [fx, top_y, fz + 1.0], normal_idx: 4, uv: [0.0, 0.0], tex_layer, ao: ao[3] },
                        ];
                        Self::push_quad(&mut vertices, &mut indices, quad, ao);
                    }

                    // Face 5: -Z (North, normal = (0, 0, -1))
                    if Self::is_face_visible(world.get_block(gx, gy, gz - 1), block) {
                        let tex_layer = block.texture_layer(5);
                        let ao = [
                            (Self::vertex_ao(world, gx + 1, gy, gz - 1, gx, gy - 1, gz - 1, gx + 1, gy - 1, gz - 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx - 1, gy, gz - 1, gx, gy - 1, gz - 1, gx - 1, gy - 1, gz - 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx - 1, gy, gz - 1, gx, gy + 1, gz - 1, gx - 1, gy + 1, gz - 1) + emit_boost).min(1.6),
                            (Self::vertex_ao(world, gx + 1, gy, gz - 1, gx, gy + 1, gz - 1, gx + 1, gy + 1, gz - 1) + emit_boost).min(1.6),
                        ];
                        let quad = [
                            BlockVertex { position: [fx + 1.0, fy, fz], normal_idx: 5, uv: [0.0, 1.0], tex_layer, ao: ao[0] },
                            BlockVertex { position: [fx, fy, fz], normal_idx: 5, uv: [1.0, 1.0], tex_layer, ao: ao[1] },
                            BlockVertex { position: [fx, top_y, fz], normal_idx: 5, uv: [1.0, 0.0], tex_layer, ao: ao[2] },
                            BlockVertex { position: [fx + 1.0, top_y, fz], normal_idx: 5, uv: [0.0, 0.0], tex_layer, ao: ao[3] },
                        ];
                        Self::push_quad(&mut vertices, &mut indices, quad, ao);
                    }
                }
            }
        }

        ChunkMesh { vertices, indices }
    }

    fn push_torch_mesh(
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        fx: f32,
        fy: f32,
        fz: f32,
    ) {
        let x0 = fx + 0.42;
        let x1 = fx + 0.58;
        let y0 = fy;
        let y1 = fy + 0.65;
        let z0 = fz + 0.42;
        let z1 = fz + 0.58;
        let tl_side = BlockType::Planks.texture_layer(0);
        let tl_top = BlockType::Lava.texture_layer(2);
        let ao = 1.55;

        let faces: [([f32; 3], [f32; 3], [f32; 3], [f32; 3], u32, u32); 5] = [
            ([x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1], 0, tl_side),
            ([x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0], 1, tl_side),
            ([x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0], 2, tl_top),
            ([x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1], 4, tl_side),
            ([x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0], 5, tl_side),
        ];

        for (p0, p1, p2, p3, n_idx, tl) in faces {
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: p0, normal_idx: n_idx, uv: [0.4, 1.0], tex_layer: tl, ao },
                BlockVertex { position: p1, normal_idx: n_idx, uv: [0.6, 1.0], tex_layer: tl, ao },
                BlockVertex { position: p2, normal_idx: n_idx, uv: [0.6, 0.2], tex_layer: tl, ao },
                BlockVertex { position: p3, normal_idx: n_idx, uv: [0.4, 0.2], tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    #[inline(always)]
    fn local_light_boost(world: &World, gx: i32, gy: i32, gz: i32, block: BlockType) -> f32 {
        if block.light_emission() > 0 {
            return 0.65;
        }
        let mut max_emit = 0u8;
        for (dx, dy, dz) in [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
            (1, 1, 0),
            (-1, 1, 0),
            (0, 1, 1),
            (0, 1, -1),
        ] {
            let e = world.get_block(gx + dx, gy + dy, gz + dz).light_emission();
            if e > max_emit {
                max_emit = e;
            }
        }
        if max_emit > 0 {
            (max_emit as f32 / 15.0) * 0.45
        } else {
            0.0
        }
    }

    #[inline(always)]
    fn is_face_visible(neighbor: BlockType, current: BlockType) -> bool {
        if neighbor == BlockType::Air || neighbor == BlockType::Torch {
            true
        } else if !neighbor.is_opaque() && current.is_opaque() {
            true
        } else if !neighbor.is_opaque() && !current.is_opaque() {
            neighbor != current
        } else {
            false
        }
    }

    #[inline(always)]
    fn vertex_ao(
        world: &World,
        s1_x: i32, s1_y: i32, s1_z: i32,
        s2_x: i32, s2_y: i32, s2_z: i32,
        c_x: i32, c_y: i32, c_z: i32,
    ) -> f32 {
        let side1 = world.get_block(s1_x, s1_y, s1_z).is_opaque();
        let side2 = world.get_block(s2_x, s2_y, s2_z).is_opaque();
        let corner = world.get_block(c_x, c_y, c_z).is_opaque();

        let count = if side1 && side2 {
            0
        } else {
            3 - (side1 as i32 + side2 as i32 + corner as i32)
        };

        match count {
            0 => 0.40,
            1 => 0.60,
            2 => 0.80,
            _ => 1.00,
        }
    }

    #[inline(always)]
    fn push_quad(
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        quad: [BlockVertex; 4],
        ao: [f32; 4],
    ) {
        let base_idx = vertices.len() as u32;
        vertices.extend_from_slice(&quad);

        if ao[0] + ao[2] > ao[1] + ao[3] {
            indices.extend_from_slice(&[
                base_idx,
                base_idx + 1,
                base_idx + 2,
                base_idx,
                base_idx + 2,
                base_idx + 3,
            ]);
        } else {
            indices.extend_from_slice(&[
                base_idx + 1,
                base_idx + 2,
                base_idx + 3,
                base_idx + 1,
                base_idx + 3,
                base_idx,
            ]);
        }
    }
}
