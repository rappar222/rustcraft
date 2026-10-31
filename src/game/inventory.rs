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

#[derive(Debug, Clone, Copy)]
pub struct CraftingRecipe {
    pub label: &'static str,
    pub input: BlockType,
    pub input_count: u32,
    pub output: BlockType,
    pub output_count: u32,
}

#[derive(Clone)]
pub struct Inventory {
    pub hotbar: [ItemStack; 9],
    pub storage: [ItemStack; 27],
    pub selected_slot: usize, // 0..8 active hotbar slot
    pub held_slot: Option<usize>, // 0..36 selected slot in inventory screen for swapping
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

    pub const RECIPES: [CraftingRecipe; 4] = [
        CraftingRecipe {
            label: "1 Wood -> 4 Planks",
            input: BlockType::Wood,
            input_count: 1,
            output: BlockType::Planks,
            output_count: 4,
        },
        CraftingRecipe {
            label: "1 Sand -> 1 Glass",
            input: BlockType::Sand,
            input_count: 1,
            output: BlockType::Glass,
            output_count: 1,
        },
        CraftingRecipe {
            label: "1 Cobble -> 1 Stone",
            input: BlockType::Cobblestone,
            input_count: 1,
            output: BlockType::Stone,
            output_count: 1,
        },
        CraftingRecipe {
            label: "2 Dirt -> 1 Grass",
            input: BlockType::Dirt,
            input_count: 2,
            output: BlockType::Grass,
            output_count: 1,
        },
    ];

    pub fn new_survival() -> Self {
        Self {
            hotbar: [ItemStack::EMPTY; 9],
            storage: [ItemStack::EMPTY; 27],
            selected_slot: 0,
            held_slot: None,
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
            storage: [ItemStack::EMPTY; 27],
            selected_slot: 0,
            held_slot: None,
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

    pub fn cycle_slot(&mut self, delta: i32) {
        let next = (self.selected_slot as i32 + delta).rem_euclid(9);
        self.selected_slot = next as usize;
    }

    pub fn set_slot(&mut self, slot: usize, block: BlockType, count: u32) {
        if slot < 9 {
            self.hotbar[slot] = ItemStack::new(block, count);
        }
    }

    /// Combined index: 0..9 = hotbar[0..9], 9..36 = storage[0..27]
    pub fn get_combined_slot(&self, idx: usize) -> ItemStack {
        if idx < 9 {
            self.hotbar[idx]
        } else if idx < 36 {
            self.storage[idx - 9]
        } else {
            ItemStack::EMPTY
        }
    }

    pub fn set_combined_slot(&mut self, idx: usize, stack: ItemStack) {
        if idx < 9 {
            self.hotbar[idx] = stack;
        } else if idx < 36 {
            self.storage[idx - 9] = stack;
        }
    }

    /// Swap or merge two slots (0..36)
    pub fn interact_combined_slots(&mut self, slot_a: usize, slot_b: usize) {
        if slot_a == slot_b || slot_a >= 36 || slot_b >= 36 {
            return;
        }

        let a = self.get_combined_slot(slot_a);
        let b = self.get_combined_slot(slot_b);

        if !a.is_empty() && a.block == b.block && b.count < 64 {
            let space = 64 - b.count;
            let transfer = space.min(a.count);
            self.set_combined_slot(slot_b, ItemStack::new(b.block, b.count + transfer));
            self.set_combined_slot(slot_a, ItemStack::new(a.block, a.count - transfer));
        } else {
            self.set_combined_slot(slot_a, b);
            self.set_combined_slot(slot_b, a);
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

    /// Add items to the inventory (hotbar first, then 27-slot backpack storage).
    /// Returns the number of items that were successfully added.
    pub fn add_item_partial(&mut self, block: BlockType, mut amount: u32) -> u32 {
        if block == BlockType::Air || amount == 0 {
            return amount;
        }
        let initial = amount;

        // 1. Try to stack with existing hotbar slots of the same block
        for slot in &mut self.hotbar {
            if slot.block == block && slot.count < 64 {
                let space = 64 - slot.count;
                let add = space.min(amount);
                slot.count += add;
                amount -= add;
                if amount == 0 {
                    return initial;
                }
            }
        }

        // 2. Try to stack with existing storage slots of the same block
        for slot in &mut self.storage {
            if slot.block == block && slot.count < 64 {
                let space = 64 - slot.count;
                let add = space.min(amount);
                slot.count += add;
                amount -= add;
                if amount == 0 {
                    return initial;
                }
            }
        }

        // 3. Try to put remaining amount into an empty hotbar slot
        for slot in &mut self.hotbar {
            if slot.is_empty() {
                let add = 64.min(amount);
                *slot = ItemStack::new(block, add);
                amount -= add;
                if amount == 0 {
                    return initial;
                }
            }
        }

        // 4. Try to put remaining amount into an empty storage slot
        for slot in &mut self.storage {
            if slot.is_empty() {
                let add = 64.min(amount);
                *slot = ItemStack::new(block, add);
                amount -= add;
                if amount == 0 {
                    return initial;
                }
            }
        }

        initial - amount
    }

    /// Add an item to the inventory; returns true if at least 1 (or all) items were stored.
    pub fn add_item(&mut self, block: BlockType, amount: u32) -> bool {
        if block == BlockType::Air || amount == 0 {
            return true;
        }
        self.add_item_partial(block, amount) == amount
    }

    pub fn count_block(&self, block: BlockType) -> u32 {
        let mut total = 0;
        for s in &self.hotbar {
            if s.block == block {
                total += s.count;
            }
        }
        for s in &self.storage {
            if s.block == block {
                total += s.count;
            }
        }
        total
    }

    pub fn remove_block(&mut self, block: BlockType, mut amount: u32) -> bool {
        if self.count_block(block) < amount {
            return false;
        }

        // Remove from storage first, then hotbar
        for s in &mut self.storage {
            if s.block == block && s.count > 0 {
                let take = s.count.min(amount);
                s.count -= take;
                amount -= take;
                if s.count == 0 {
                    *s = ItemStack::EMPTY;
                }
                if amount == 0 {
                    return true;
                }
            }
        }

        for s in &mut self.hotbar {
            if s.block == block && s.count > 0 {
                let take = s.count.min(amount);
                s.count -= take;
                amount -= take;
                if s.count == 0 {
                    *s = ItemStack::EMPTY;
                }
                if amount == 0 {
                    return true;
                }
            }
        }

        amount == 0
    }

    pub fn can_craft(&self, recipe_idx: usize) -> bool {
        if recipe_idx >= Self::RECIPES.len() {
            return false;
        }
        let r = &Self::RECIPES[recipe_idx];
        self.count_block(r.input) >= r.input_count
    }

    pub fn craft(&mut self, recipe_idx: usize) -> bool {
        if !self.can_craft(recipe_idx) {
            return false;
        }
        let r = Self::RECIPES[recipe_idx];
        // Check if output fits by testing on a clone
        let mut test_inv = self.clone();
        if test_inv.remove_block(r.input, r.input_count)
            && test_inv.add_item(r.output, r.output_count)
        {
            *self = test_inv;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_items_and_overflow_to_storage() {
        let mut inv = Inventory::new_survival();
        for _ in 0..10 {
            assert!(inv.add_item(BlockType::Dirt, 1));
        }
        assert_eq!(inv.hotbar[0].block, BlockType::Dirt);
        assert_eq!(inv.hotbar[0].count, 10);

        for _ in 0..60 {
            assert!(inv.add_item(BlockType::Dirt, 1));
        }
        // Slot 0 full at 64, slot 1 should have 6
        assert_eq!(inv.hotbar[0].count, 64);
        assert_eq!(inv.hotbar[1].block, BlockType::Dirt);
        assert_eq!(inv.hotbar[1].count, 6);

        // Fill all 9 hotbar slots and verify overflow into 27-slot storage
        let mut inv2 = Inventory::new_survival();
        for i in 0..15 {
            assert!(inv2.add_item(BlockType::Cobblestone, 64), "Failed at stack {}", i);
        }
        assert_eq!(inv2.hotbar[8].count, 64);
        assert_eq!(inv2.storage[0].count, 64);
        assert_eq!(inv2.storage[5].count, 64);
    }

    #[test]
    fn test_crafting_and_swapping() {
        let mut inv = Inventory::new_survival();
        inv.add_item(BlockType::Wood, 2);
        assert!(inv.can_craft(0)); // 1 Wood -> 4 Planks
        assert!(inv.craft(0));
        assert_eq!(inv.count_block(BlockType::Wood), 1);
        assert_eq!(inv.count_block(BlockType::Planks), 4);

        // Swap hotbar[0] (Wood x1) and hotbar[1] (Planks x4)
        inv.interact_combined_slots(0, 1);
        assert_eq!(inv.hotbar[0].block, BlockType::Planks);
        assert_eq!(inv.hotbar[1].block, BlockType::Wood);
    }
}
