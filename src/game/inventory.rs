use crate::voxel::block::BlockType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemStack {
    pub block: BlockType,
    pub count: u32,
}

impl ItemStack {
    pub const EMPTY: Self = Self {
        block: BlockType::Air,
        count: 0,
    };

    pub fn new(block: BlockType, count: u32) -> Self {
        if block == BlockType::Air || count == 0 {
            Self::EMPTY
        } else {
            Self { block, count }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.block == BlockType::Air || self.count == 0
    }
}

pub struct Inventory {
    pub hotbar: [ItemStack; 9],
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

    pub fn new_survival() -> Self {
        Self {
            hotbar: [ItemStack::EMPTY; 9],
            selected_slot: 0,
        }
    }

    pub fn new_creative() -> Self {
        Self {
            hotbar: [
                ItemStack::new(BlockType::Grass, 64),
                ItemStack::new(BlockType::Dirt, 64),
                ItemStack::new(BlockType::Stone, 64),
                ItemStack::new(BlockType::Cobblestone, 64),
                ItemStack::new(BlockType::Wood, 64),
                ItemStack::new(BlockType::Leaves, 64),
                ItemStack::new(BlockType::Sand, 64),
                ItemStack::new(BlockType::Planks, 64),
                ItemStack::new(BlockType::Glass, 64),
            ],
            selected_slot: 0,
        }
    }

    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::new_survival()
    }

    pub fn selected_stack(&self) -> ItemStack {
        self.hotbar[self.selected_slot]
    }

    pub fn selected_block(&self) -> BlockType {
        self.hotbar[self.selected_slot].block
    }

    pub fn select_slot(&mut self, slot: usize) {
        if slot < 9 {
            self.selected_slot = slot;
        }
    }

    pub fn set_slot(&mut self, slot: usize, block: BlockType, count: u32) {
        if slot < 9 {
            self.hotbar[slot] = ItemStack::new(block, count);
        }
    }

    pub fn can_place_selected(&self, is_creative: bool) -> bool {
        if is_creative {
            self.selected_block() != BlockType::Air
        } else {
            !self.selected_stack().is_empty()
        }
    }

    pub fn consume_selected(&mut self, is_creative: bool) -> bool {
        if is_creative {
            return self.selected_block() != BlockType::Air;
        }

        let slot = self.selected_slot;
        if self.hotbar[slot].count > 0 && self.hotbar[slot].block != BlockType::Air {
            self.hotbar[slot].count -= 1;
            if self.hotbar[slot].count == 0 {
                self.hotbar[slot] = ItemStack::EMPTY;
            }
            true
        } else {
            false
        }
    }

    /// Add an item to the inventory (stacking with existing matching block or first empty slot)
    pub fn add_item(&mut self, block: BlockType, mut amount: u32) -> bool {
        if block == BlockType::Air || amount == 0 {
            return true;
        }

        // 1. Try to stack with existing slots of the same block
        for slot in &mut self.hotbar {
            if slot.block == block && slot.count < 64 {
                let space = 64 - slot.count;
                let add = space.min(amount);
                slot.count += add;
                amount -= add;
                if amount == 0 {
                    return true;
                }
            }
        }

        // 2. Try to put remaining amount into an empty slot
        for slot in &mut self.hotbar {
            if slot.is_empty() {
                let add = 64.min(amount);
                *slot = ItemStack::new(block, add);
                amount -= add;
                if amount == 0 {
                    return true;
                }
            }
        }

        amount == 0 // returns true if all items were stored
    }
}
