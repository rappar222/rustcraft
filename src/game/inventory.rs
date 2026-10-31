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
    pub input1: BlockType,
    pub count1: u32,
    pub input2: Option<(BlockType, u32)>,
    pub output: BlockType,
    pub output_count: u32,
    pub requires_table: bool,
    pub requires_furnace: bool,
}

#[derive(Clone)]
pub struct Inventory {
    pub hotbar: [ItemStack; 9],
    pub storage: [ItemStack; 27],
    pub selected_slot: usize,     // 0..8 active hotbar slot
    pub held_slot: Option<usize>, // 0..36 selected slot in inventory screen for swapping
}

impl Inventory {
    pub const PALETTE: [BlockType; 28] = [
        BlockType::Grass,
        BlockType::Dirt,
        BlockType::Stone,
        BlockType::Cobblestone,
        BlockType::Wood,
        BlockType::BirchWood,
        BlockType::SpruceWood,
        BlockType::Leaves,
        BlockType::Sand,
        BlockType::Gravel,
        BlockType::Snow,
        BlockType::Ice,
        BlockType::Planks,
        BlockType::Brick,
        BlockType::Bookshelf,
        BlockType::Glass,
        BlockType::Glowstone,
        BlockType::Obsidian,
        BlockType::Torch,
        BlockType::CraftingTable,
        BlockType::Furnace,
        BlockType::TNT,
        BlockType::Water,
        BlockType::Lava,
        BlockType::DiamondPickaxe,
        BlockType::DiamondSword,
        BlockType::DiamondAxe,
        BlockType::CookedPorkchop,
    ];

    pub const RECIPES: [CraftingRecipe; 18] = [
        // Basic Hand Recipes (Always available)
        CraftingRecipe {
            label: "1 Log -> 4 Planks",
            input1: BlockType::Wood,
            count1: 1,
            input2: None,
            output: BlockType::Planks,
            output_count: 4,
            requires_table: false,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "2 Planks -> 4 Sticks",
            input1: BlockType::Planks,
            count1: 2,
            input2: None,
            output: BlockType::Stick,
            output_count: 4,
            requires_table: false,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "4 Planks -> WorkBench",
            input1: BlockType::Planks,
            count1: 4,
            input2: None,
            output: BlockType::CraftingTable,
            output_count: 1,
            requires_table: false,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "1 Coal+1 Stick->4 Torch",
            input1: BlockType::Coal,
            count1: 1,
            input2: Some((BlockType::Stick, 1)),
            output: BlockType::Torch,
            output_count: 4,
            requires_table: false,
            requires_furnace: false,
        },
        // Workbench Tool & Weapon Recipes
        CraftingRecipe {
            label: "3 Plank+2 Stick->WdPick",
            input1: BlockType::Planks,
            count1: 3,
            input2: Some((BlockType::Stick, 2)),
            output: BlockType::WoodPickaxe,
            output_count: 1,
            requires_table: true,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "3 Cobbl+2 Stick->StPick",
            input1: BlockType::Cobblestone,
            count1: 3,
            input2: Some((BlockType::Stick, 2)),
            output: BlockType::StonePickaxe,
            output_count: 1,
            requires_table: true,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "3 Iron+2 Stick->IrnPick",
            input1: BlockType::IronIngot,
            count1: 3,
            input2: Some((BlockType::Stick, 2)),
            output: BlockType::IronPickaxe,
            output_count: 1,
            requires_table: true,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "3 Diam+2 Stick->DiaPick",
            input1: BlockType::Diamond,
            count1: 3,
            input2: Some((BlockType::Stick, 2)),
            output: BlockType::DiamondPickaxe,
            output_count: 1,
            requires_table: true,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "2 Plank+1 Stick->WdSwrd",
            input1: BlockType::Planks,
            count1: 2,
            input2: Some((BlockType::Stick, 1)),
            output: BlockType::WoodSword,
            output_count: 1,
            requires_table: true,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "2 Cobbl+1 Stick->StSwrd",
            input1: BlockType::Cobblestone,
            count1: 2,
            input2: Some((BlockType::Stick, 1)),
            output: BlockType::StoneSword,
            output_count: 1,
            requires_table: true,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "2 Iron+1 Stick->IrnSwrd",
            input1: BlockType::IronIngot,
            count1: 2,
            input2: Some((BlockType::Stick, 1)),
            output: BlockType::IronSword,
            output_count: 1,
            requires_table: true,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "2 Diam+1 Stick->DiaSwrd",
            input1: BlockType::Diamond,
            count1: 2,
            input2: Some((BlockType::Stick, 1)),
            output: BlockType::DiamondSword,
            output_count: 1,
            requires_table: true,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "3 Cobbl+2 Stick->StnAxe",
            input1: BlockType::Cobblestone,
            count1: 3,
            input2: Some((BlockType::Stick, 2)),
            output: BlockType::StoneAxe,
            output_count: 1,
            requires_table: true,
            requires_furnace: false,
        },
        CraftingRecipe {
            label: "8 Cobble -> 1 Furnace",
            input1: BlockType::Cobblestone,
            count1: 8,
            input2: None,
            output: BlockType::Furnace,
            output_count: 1,
            requires_table: true,
            requires_furnace: false,
        },
        // Furnace Smelting & Advanced Recipes
        CraftingRecipe {
            label: "Smelt IronOre -> Ingot",
            input1: BlockType::IronOre,
            count1: 1,
            input2: Some((BlockType::Coal, 1)),
            output: BlockType::IronIngot,
            output_count: 1,
            requires_table: false,
            requires_furnace: true,
        },
        CraftingRecipe {
            label: "Smelt GoldOre -> Ingot",
            input1: BlockType::GoldOre,
            count1: 1,
            input2: Some((BlockType::Coal, 1)),
            output: BlockType::GoldIngot,
            output_count: 1,
            requires_table: false,
            requires_furnace: true,
        },
        CraftingRecipe {
            label: "Cook Porkchop (+8 Food)",
            input1: BlockType::RawPorkchop,
            count1: 1,
            input2: Some((BlockType::Coal, 1)),
            output: BlockType::CookedPorkchop,
            output_count: 1,
            requires_table: false,
            requires_furnace: true,
        },
        CraftingRecipe {
            label: "Smelt Sand -> 1 Glass",
            input1: BlockType::Sand,
            count1: 1,
            input2: None,
            output: BlockType::Glass,
            output_count: 1,
            requires_table: false,
            requires_furnace: true,
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
                ItemStack::new(BlockType::Stone, 64),
                ItemStack::new(BlockType::Wood, 64),
                ItemStack::new(BlockType::Planks, 64),
                ItemStack::new(BlockType::Glass, 64),
                ItemStack::new(BlockType::Torch, 64),
                ItemStack::new(BlockType::Glowstone, 64),
                ItemStack::new(BlockType::TNT, 64),
                ItemStack::new(BlockType::DiamondSword, 1),
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
            self.hotbar[slot] = ItemStack::new(block, count.min(block.max_stack_size()));
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
        let max_stack = b.block.max_stack_size();

        if !a.is_empty() && a.block == b.block && b.count < max_stack {
            let space = max_stack - b.count;
            let transfer = space.min(a.count);
            self.set_combined_slot(slot_b, ItemStack::new(b.block, b.count + transfer));
            self.set_combined_slot(slot_a, ItemStack::new(a.block, a.count - transfer));
        } else {
            self.set_combined_slot(slot_a, b);
            self.set_combined_slot(slot_b, a);
        }
    }

    pub fn can_place_selected(&self, is_creative: bool) -> bool {
        let block = self.selected_block();
        if !block.is_placeable() {
            return false;
        }
        if is_creative {
            true
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

    /// Toss 1 item from the currently selected hotbar slot
    pub fn drop_selected_one(&mut self) -> Option<BlockType> {
        let slot = self.selected_slot;
        let stack = self.hotbar[slot];
        if !stack.is_empty() {
            let block = stack.block;
            self.hotbar[slot].count -= 1;
            if self.hotbar[slot].count == 0 {
                self.hotbar[slot] = ItemStack::EMPTY;
            }
            Some(block)
        } else {
            None
        }
    }

    /// Add items to the inventory (hotbar first, then 27-slot backpack storage).
    /// Returns the number of items that were successfully added.
    pub fn add_item_partial(&mut self, block: BlockType, mut amount: u32) -> u32 {
        if block == BlockType::Air || amount == 0 {
            return amount;
        }
        let initial = amount;
        let max_stack = block.max_stack_size();

        // 1. Try to stack with existing hotbar slots of the same block
        if max_stack > 1 {
            for slot in &mut self.hotbar {
                if slot.block == block && slot.count < max_stack {
                    let space = max_stack - slot.count;
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
                if slot.block == block && slot.count < max_stack {
                    let space = max_stack - slot.count;
                    let add = space.min(amount);
                    slot.count += add;
                    amount -= add;
                    if amount == 0 {
                        return initial;
                    }
                }
            }
        }

        // 3. Try to put remaining amount into an empty hotbar slot
        for slot in &mut self.hotbar {
            if slot.is_empty() {
                let add = max_stack.min(amount);
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
                let add = max_stack.min(amount);
                *slot = ItemStack::new(block, add);
                amount -= add;
                if amount == 0 {
                    return initial;
                }
            }
        }

        initial - amount
    }

    pub fn add_item(&mut self, block: BlockType, amount: u32) -> bool {
        if block == BlockType::Air || amount == 0 {
            return true;
        }
        self.add_item_partial(block, amount) == amount
    }

    fn matches_ingredient(slot_block: BlockType, req: BlockType) -> bool {
        if req == BlockType::Wood {
            matches!(
                slot_block,
                BlockType::Wood | BlockType::BirchWood | BlockType::SpruceWood
            )
        } else {
            slot_block == req
        }
    }

    pub fn count_block(&self, block: BlockType) -> u32 {
        let mut total = 0;
        for s in &self.hotbar {
            if Self::matches_ingredient(s.block, block) {
                total += s.count;
            }
        }
        for s in &self.storage {
            if Self::matches_ingredient(s.block, block) {
                total += s.count;
            }
        }
        total
    }

    pub fn remove_block(&mut self, block: BlockType, mut amount: u32) -> bool {
        if self.count_block(block) < amount {
            return false;
        }

        for s in &mut self.storage {
            if Self::matches_ingredient(s.block, block) && s.count > 0 {
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
            if Self::matches_ingredient(s.block, block) && s.count > 0 {
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

    pub fn can_craft(&self, recipe_idx: usize, near_table: bool, near_furnace: bool) -> bool {
        if recipe_idx >= Self::RECIPES.len() {
            return false;
        }
        let r = &Self::RECIPES[recipe_idx];
        let has_table = near_table || self.count_block(BlockType::CraftingTable) > 0;
        let has_furnace = near_furnace || self.count_block(BlockType::Furnace) > 0;

        if r.requires_table && !has_table {
            return false;
        }
        if r.requires_furnace && !has_furnace {
            return false;
        }

        if self.count_block(r.input1) < r.count1 {
            return false;
        }
        if let Some((in2, cnt2)) = r.input2 {
            if self.count_block(in2) < cnt2 {
                return false;
            }
        }
        true
    }

    pub fn craft(&mut self, recipe_idx: usize, near_table: bool, near_furnace: bool) -> bool {
        if !self.can_craft(recipe_idx, near_table, near_furnace) {
            return false;
        }
        let r = Self::RECIPES[recipe_idx];
        let mut test_inv = self.clone();
        if !test_inv.remove_block(r.input1, r.count1) {
            return false;
        }
        if let Some((in2, cnt2)) = r.input2 {
            if !test_inv.remove_block(in2, cnt2) {
                return false;
            }
        }
        if test_inv.add_item(r.output, r.output_count) {
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
        assert_eq!(inv.hotbar[0].count, 64);
        assert_eq!(inv.hotbar[1].block, BlockType::Dirt);
        assert_eq!(inv.hotbar[1].count, 6);

        let mut inv2 = Inventory::new_survival();
        for i in 0..15 {
            assert!(inv2.add_item(BlockType::Cobblestone, 64), "Failed at stack {}", i);
        }
        assert_eq!(inv2.hotbar[8].count, 64);
        assert_eq!(inv2.storage[0].count, 64);
        assert_eq!(inv2.storage[5].count, 64);
    }

    #[test]
    fn test_crafting_progression_and_swapping() {
        let mut inv = Inventory::new_survival();
        // SpruceWood also counts as Log for Planks!
        inv.add_item(BlockType::SpruceWood, 3);
        assert!(inv.can_craft(0, false, false)); // 1 Log -> 4 Planks
        assert!(inv.craft(0, false, false));
        assert!(inv.craft(0, false, false));
        assert_eq!(inv.count_block(BlockType::Planks), 8);

        // Craft Sticks (2 Planks -> 4 Sticks)
        assert!(inv.craft(1, false, false));
        assert_eq!(inv.count_block(BlockType::Stick), 4);

        // Craft CraftingTable (4 Planks -> 1 CraftingTable)
        assert!(inv.craft(2, false, false));
        assert_eq!(inv.count_block(BlockType::CraftingTable), 1);

        // Now CraftingTable recipes unlock! Craft Stone Pickaxe
        inv.add_item(BlockType::Cobblestone, 3);
        assert!(inv.can_craft(5, false, false));
        assert!(inv.craft(5, false, false));
        assert_eq!(inv.count_block(BlockType::StonePickaxe), 1);
    }
}
