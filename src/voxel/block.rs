#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum BlockType {
    #[default]
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Cobblestone = 4,
    Wood = 5,
    Leaves = 6,
    Sand = 7,
    Planks = 8,
    Bedrock = 9,
    Glass = 10,
}

impl BlockType {
    #[allow(dead_code)]
    pub const COUNT: usize = 11;

    #[inline(always)]
    pub fn is_opaque(self) -> bool {
        match self {
            BlockType::Air => false,
            BlockType::Leaves => false, // Transparent / cutout
            BlockType::Glass => false,  // Transparent
            _ => true,
        }
    }

    #[inline(always)]
    pub fn is_solid(self) -> bool {
        match self {
            BlockType::Air => false,
            _ => true,
        }
    }

    // Direction indices: 0: +X (East), 1: -X (West), 2: +Y (Top), 3: -Y (Bottom), 4: +Z (South), 5: -Z (North)
    #[inline(always)]
    pub fn texture_layer(self, face_idx: usize) -> u32 {
        match self {
            BlockType::Air => 0,
            BlockType::Grass => match face_idx {
                2 => 0, // Grass Top
                3 => 2, // Dirt Bottom
                _ => 1, // Grass Side
            },
            BlockType::Dirt => 2,
            BlockType::Stone => 3,
            BlockType::Cobblestone => 4,
            BlockType::Wood => match face_idx {
                2 | 3 => 6, // Log top/bottom
                _ => 5,     // Log side
            },
            BlockType::Leaves => 7,
            BlockType::Sand => 8,
            BlockType::Planks => 9,
            BlockType::Bedrock => 10,
            BlockType::Glass => 11,
        }
    }
}
