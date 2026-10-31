use crate::voxel::block::BlockType;

pub struct Inventory {
    pub hotbar: [BlockType; 9],
    pub selected_slot: usize, // 0..8
}

impl Inventory {
    pub const PALETTE: [BlockType; 10] = [
        BlockType::Grass,
        BlockType::Dirt,
        BlockType::Stone,
        BlockType::Cobblestone,
        BlockType::Wood,
        BlockType::Leaves,
        BlockType::Sand,
        BlockType::Planks,
        BlockType::Glass,
        BlockType::Bedrock,
    ];

    pub fn new() -> Self {
        Self {
            hotbar: [
                BlockType::Grass,
                BlockType::Dirt,
                BlockType::Stone,
                BlockType::Cobblestone,
                BlockType::Wood,
                BlockType::Leaves,
                BlockType::Sand,
                BlockType::Planks,
                BlockType::Glass,
            ],
            selected_slot: 0,
        }
    }

    pub fn selected_block(&self) -> BlockType {
        self.hotbar[self.selected_slot]
    }

    pub fn select_slot(&mut self, slot: usize) {
        if slot < 9 {
            self.selected_slot = slot;
        }
    }

    pub fn set_slot(&mut self, slot: usize, block: BlockType) {
        if slot < 9 {
            self.hotbar[slot] = block;
        }
    }
}
