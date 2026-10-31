use super::block::BlockType;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 128;
pub const CHUNK_DEPTH: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    #[inline(always)]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    #[inline(always)]
    pub const fn from_block_coords(bx: i32, bz: i32) -> Self {
        Self {
            x: bx.div_euclid(CHUNK_WIDTH as i32),
            z: bz.div_euclid(CHUNK_DEPTH as i32),
        }
    }
}

pub struct Chunk {
    pub pos: ChunkPos,
    pub blocks: Box<[BlockType; CHUNK_VOLUME]>,
    pub is_dirty: bool,
}

impl Chunk {
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            blocks: Box::new([BlockType::Air; CHUNK_VOLUME]),
            is_dirty: true,
        }
    }

    #[inline(always)]
    pub const fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_WIDTH + y * (CHUNK_WIDTH * CHUNK_DEPTH)
    }

    #[inline(always)]
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            self.blocks[Self::index(x, y, z)]
        } else {
            BlockType::Air
        }
    }

    #[inline(always)]
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            let idx = Self::index(x, y, z);
            if self.blocks[idx] != block {
                self.blocks[idx] = block;
                self.is_dirty = true;
            }
        }
    }
}
