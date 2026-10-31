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
            BlockType::Leaves => false,
            BlockType::Glass => false,
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

    /// Time in seconds required to break the block in Survival mode
    #[inline(always)]
    pub fn break_time(self) -> f32 {
        match self {
            BlockType::Air => f32::INFINITY,
            BlockType::Leaves => 0.25,
            BlockType::Glass => 0.35,
            BlockType::Sand => 0.50,
            BlockType::Dirt => 0.60,
            BlockType::Grass => 0.60,
            BlockType::Planks => 1.20,
            BlockType::Wood => 1.50,
            BlockType::Cobblestone => 1.80,
            BlockType::Stone => 2.00,
            BlockType::Bedrock => f32::INFINITY, // Unbreakable!
        }
    }

    /// What block type is dropped when this block is destroyed
    #[inline(always)]
    pub fn drop_item(self) -> Option<BlockType> {
        match self {
            BlockType::Air => None,
            BlockType::Grass => Some(BlockType::Dirt), // Grass drops Dirt
            BlockType::Stone => Some(BlockType::Cobblestone), // Stone drops Cobblestone
            BlockType::Cobblestone => Some(BlockType::Cobblestone),
            BlockType::Dirt => Some(BlockType::Dirt),
            BlockType::Wood => Some(BlockType::Wood),
            BlockType::Planks => Some(BlockType::Planks),
            BlockType::Sand => Some(BlockType::Sand),
            BlockType::Leaves => Some(BlockType::Leaves),
            BlockType::Glass => None, // Glass shatters without drop
            BlockType::Bedrock => None,
        }
    }
}
