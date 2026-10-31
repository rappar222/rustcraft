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
    Water = 11,
    Lava = 12,
    CoalOre = 13,
    IronOre = 14,
    GoldOre = 15,
    DiamondOre = 16,
    Gravel = 17,
    Snow = 18,
    Ice = 19,
    Cactus = 20,
    CraftingTable = 21,
    Furnace = 22,
    Torch = 23,
    Brick = 24,
    TNT = 25,
    Obsidian = 26,
    Glowstone = 27,
    Bookshelf = 28,
    BirchWood = 29,
    SpruceWood = 30,

    // Non-block holdable items, materials, tools, weapons, and food
    Stick = 31,
    Coal = 32,
    IronIngot = 33,
    GoldIngot = 34,
    Diamond = 35,
    WoodPickaxe = 36,
    StonePickaxe = 37,
    IronPickaxe = 38,
    DiamondPickaxe = 39,
    WoodAxe = 40,
    StoneAxe = 41,
    IronAxe = 42,
    DiamondAxe = 43,
    WoodShovel = 44,
    StoneShovel = 45,
    IronShovel = 46,
    DiamondShovel = 47,
    WoodSword = 48,
    StoneSword = 49,
    IronSword = 50,
    DiamondSword = 51,
    Apple = 52,
    Bread = 53,
    RawPorkchop = 54,
    CookedPorkchop = 55,
    RottenFlesh = 56,
}

impl BlockType {
    #[allow(dead_code)]
    pub const COUNT: usize = 57;

    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => BlockType::Grass,
            2 => BlockType::Dirt,
            3 => BlockType::Stone,
            4 => BlockType::Cobblestone,
            5 => BlockType::Wood,
            6 => BlockType::Leaves,
            7 => BlockType::Sand,
            8 => BlockType::Planks,
            9 => BlockType::Bedrock,
            10 => BlockType::Glass,
            11 => BlockType::Water,
            12 => BlockType::Lava,
            13 => BlockType::CoalOre,
            14 => BlockType::IronOre,
            15 => BlockType::GoldOre,
            16 => BlockType::DiamondOre,
            17 => BlockType::Gravel,
            18 => BlockType::Snow,
            19 => BlockType::Ice,
            20 => BlockType::Cactus,
            21 => BlockType::CraftingTable,
            22 => BlockType::Furnace,
            23 => BlockType::Torch,
            24 => BlockType::Brick,
            25 => BlockType::TNT,
            26 => BlockType::Obsidian,
            27 => BlockType::Glowstone,
            28 => BlockType::Bookshelf,
            29 => BlockType::BirchWood,
            30 => BlockType::SpruceWood,
            31 => BlockType::Stick,
            32 => BlockType::Coal,
            33 => BlockType::IronIngot,
            34 => BlockType::GoldIngot,
            35 => BlockType::Diamond,
            36 => BlockType::WoodPickaxe,
            37 => BlockType::StonePickaxe,
            38 => BlockType::IronPickaxe,
            39 => BlockType::DiamondPickaxe,
            40 => BlockType::WoodAxe,
            41 => BlockType::StoneAxe,
            42 => BlockType::IronAxe,
            43 => BlockType::DiamondAxe,
            44 => BlockType::WoodShovel,
            45 => BlockType::StoneShovel,
            46 => BlockType::IronShovel,
            47 => BlockType::DiamondShovel,
            48 => BlockType::WoodSword,
            49 => BlockType::StoneSword,
            50 => BlockType::IronSword,
            51 => BlockType::DiamondSword,
            52 => BlockType::Apple,
            53 => BlockType::Bread,
            54 => BlockType::RawPorkchop,
            55 => BlockType::CookedPorkchop,
            56 => BlockType::RottenFlesh,
            _ => BlockType::Air,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BlockType::Air => "Empty Hand",
            BlockType::Grass => "Grass Block",
            BlockType::Dirt => "Dirt",
            BlockType::Stone => "Stone",
            BlockType::Cobblestone => "Cobblestone",
            BlockType::Wood => "Oak Log",
            BlockType::Leaves => "Oak Leaves",
            BlockType::Sand => "Sand",
            BlockType::Planks => "Oak Planks",
            BlockType::Bedrock => "Bedrock",
            BlockType::Glass => "Glass",
            BlockType::Water => "Water",
            BlockType::Lava => "Lava",
            BlockType::CoalOre => "Coal Ore",
            BlockType::IronOre => "Iron Ore",
            BlockType::GoldOre => "Gold Ore",
            BlockType::DiamondOre => "Diamond Ore",
            BlockType::Gravel => "Gravel",
            BlockType::Snow => "Snow Block",
            BlockType::Ice => "Ice",
            BlockType::Cactus => "Cactus",
            BlockType::CraftingTable => "Crafting Table",
            BlockType::Furnace => "Furnace",
            BlockType::Torch => "Torch",
            BlockType::Brick => "Bricks",
            BlockType::TNT => "TNT",
            BlockType::Obsidian => "Obsidian",
            BlockType::Glowstone => "Glowstone",
            BlockType::Bookshelf => "Bookshelf",
            BlockType::BirchWood => "Birch Log",
            BlockType::SpruceWood => "Spruce Log",
            BlockType::Stick => "Stick",
            BlockType::Coal => "Coal",
            BlockType::IronIngot => "Iron Ingot",
            BlockType::GoldIngot => "Gold Ingot",
            BlockType::Diamond => "Diamond",
            BlockType::WoodPickaxe => "Wooden Pickaxe",
            BlockType::StonePickaxe => "Stone Pickaxe",
            BlockType::IronPickaxe => "Iron Pickaxe",
            BlockType::DiamondPickaxe => "Diamond Pickaxe",
            BlockType::WoodAxe => "Wooden Axe",
            BlockType::StoneAxe => "Stone Axe",
            BlockType::IronAxe => "Iron Axe",
            BlockType::DiamondAxe => "Diamond Axe",
            BlockType::WoodShovel => "Wooden Shovel",
            BlockType::StoneShovel => "Stone Shovel",
            BlockType::IronShovel => "Iron Shovel",
            BlockType::DiamondShovel => "Diamond Shovel",
            BlockType::WoodSword => "Wooden Sword",
            BlockType::StoneSword => "Stone Sword",
            BlockType::IronSword => "Iron Sword",
            BlockType::DiamondSword => "Diamond Sword",
            BlockType::Apple => "Apple (+4 Food)",
            BlockType::Bread => "Bread (+6 Food)",
            BlockType::RawPorkchop => "Raw Porkchop (+3 Food)",
            BlockType::CookedPorkchop => "Cooked Porkchop (+8 Food)",
            BlockType::RottenFlesh => "Rotten Flesh (+2 Food)",
        }
    }

    #[inline(always)]
    pub fn is_placeable(self) -> bool {
        let id = self as u8;
        id >= 1 && id <= 30
    }

    #[inline(always)]
    pub fn is_tool_or_weapon(self) -> bool {
        let id = self as u8;
        id >= 36 && id <= 51
    }

    #[inline(always)]
    pub fn max_stack_size(self) -> u32 {
        if self.is_tool_or_weapon() {
            1
        } else {
            64
        }
    }

    #[inline(always)]
    pub fn is_fluid(self) -> bool {
        matches!(self, BlockType::Water | BlockType::Lava)
    }

    #[inline(always)]
    pub fn is_gravity_block(self) -> bool {
        matches!(self, BlockType::Sand | BlockType::Gravel)
    }

    #[inline(always)]
    pub fn is_opaque(self) -> bool {
        match self {
            BlockType::Air
            | BlockType::Leaves
            | BlockType::Glass
            | BlockType::Water
            | BlockType::Ice
            | BlockType::Torch => false,
            _ if !self.is_placeable() => false,
            _ => true,
        }
    }

    /// Physical AABB collision solidity (Fluids and Torches can be walked through)
    #[inline(always)]
    pub fn is_solid(self) -> bool {
        match self {
            BlockType::Air | BlockType::Water | BlockType::Lava | BlockType::Torch => false,
            _ => self.is_placeable(),
        }
    }

    /// Raycast targetability (Blocks + Torches can be targeted/broken; Water/Lava/Air skipped)
    #[inline(always)]
    pub fn is_targetable(self) -> bool {
        match self {
            BlockType::Air | BlockType::Water | BlockType::Lava => false,
            _ => self.is_placeable(),
        }
    }

    /// Emissive block light level (0..15)
    #[inline(always)]
    pub fn light_emission(self) -> u8 {
        match self {
            BlockType::Lava | BlockType::Glowstone => 15,
            BlockType::Torch => 14,
            BlockType::Furnace => 10,
            _ => 0,
        }
    }

    /// Food hunger restoration value if edible
    #[inline(always)]
    pub fn food_restore(self) -> Option<f32> {
        match self {
            BlockType::Apple => Some(4.0),
            BlockType::Bread => Some(6.0),
            BlockType::RawPorkchop => Some(3.0),
            BlockType::CookedPorkchop => Some(8.0),
            BlockType::RottenFlesh => Some(2.0),
            _ => None,
        }
    }

    /// Melee attack damage dealt when hitting a mob with this item
    #[inline(always)]
    pub fn attack_damage(self) -> f32 {
        match self {
            BlockType::WoodSword => 5.0,
            BlockType::StoneSword => 6.5,
            BlockType::IronSword => 8.0,
            BlockType::DiamondSword => 10.0,
            BlockType::WoodAxe => 4.5,
            BlockType::StoneAxe => 5.5,
            BlockType::IronAxe => 7.0,
            BlockType::DiamondAxe => 8.5,
            BlockType::WoodPickaxe | BlockType::WoodShovel => 3.0,
            BlockType::StonePickaxe | BlockType::StoneShovel => 4.0,
            BlockType::IronPickaxe | BlockType::IronShovel => 5.0,
            BlockType::DiamondPickaxe | BlockType::DiamondShovel => 6.0,
            _ => 2.0, // Fist / block hit
        }
    }

    /// Mining speed multiplier when using `self` as a held tool against `target`
    #[inline(always)]
    pub fn tool_speed_multiplier(self, target: BlockType) -> f32 {
        let is_pickaxe_target = matches!(
            target,
            BlockType::Stone
                | BlockType::Cobblestone
                | BlockType::CoalOre
                | BlockType::IronOre
                | BlockType::GoldOre
                | BlockType::DiamondOre
                | BlockType::Furnace
                | BlockType::Brick
                | BlockType::Obsidian
                | BlockType::Ice
        );
        let is_axe_target = matches!(
            target,
            BlockType::Wood
                | BlockType::BirchWood
                | BlockType::SpruceWood
                | BlockType::Planks
                | BlockType::CraftingTable
                | BlockType::Bookshelf
        );
        let is_shovel_target = matches!(
            target,
            BlockType::Dirt
                | BlockType::Grass
                | BlockType::Sand
                | BlockType::Gravel
                | BlockType::Snow
        );

        match self {
            BlockType::WoodPickaxe if is_pickaxe_target => 2.5,
            BlockType::StonePickaxe if is_pickaxe_target => 4.0,
            BlockType::IronPickaxe if is_pickaxe_target => 6.5,
            BlockType::DiamondPickaxe if is_pickaxe_target => 10.0,

            BlockType::WoodAxe if is_axe_target => 2.5,
            BlockType::StoneAxe if is_axe_target => 4.0,
            BlockType::IronAxe if is_axe_target => 6.5,
            BlockType::DiamondAxe if is_axe_target => 10.0,

            BlockType::WoodShovel if is_shovel_target => 2.5,
            BlockType::StoneShovel if is_shovel_target => 4.0,
            BlockType::IronShovel if is_shovel_target => 6.5,
            BlockType::DiamondShovel if is_shovel_target => 10.0,

            BlockType::WoodSword
            | BlockType::StoneSword
            | BlockType::IronSword
            | BlockType::DiamondSword
                if matches!(target, BlockType::Leaves | BlockType::Cactus) =>
            {
                2.5
            }
            _ => 1.0,
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
                2 | 3 => 6,
                _ => 5,
            },
            BlockType::Leaves => 7,
            BlockType::Sand => 8,
            BlockType::Planks => 9,
            BlockType::Bedrock => 10,
            BlockType::Glass => 11,
            BlockType::Water => 12,
            BlockType::Lava => 13,
            BlockType::CoalOre => 14,
            BlockType::IronOre => 15,
            BlockType::GoldOre => 16,
            BlockType::DiamondOre => 17,
            BlockType::Gravel => 18,
            BlockType::Snow => match face_idx {
                2 => 19,
                3 => 2,
                _ => 20,
            },
            BlockType::Ice => 21,
            BlockType::Cactus => match face_idx {
                2 | 3 => 23,
                _ => 22,
            },
            BlockType::CraftingTable => match face_idx {
                2 => 24,
                3 => 9,
                _ => 25,
            },
            BlockType::Furnace => match face_idx {
                4 => 26, // Front grate
                _ => 27,
            },
            BlockType::Torch => 28,
            BlockType::Brick => 29,
            BlockType::TNT => match face_idx {
                2 | 3 => 31,
                _ => 30,
            },
            BlockType::Obsidian => 32,
            BlockType::Glowstone => 33,
            BlockType::Bookshelf => match face_idx {
                2 | 3 => 9,
                _ => 34,
            },
            BlockType::BirchWood => match face_idx {
                2 | 3 => 6,
                _ => 35,
            },
            BlockType::SpruceWood => match face_idx {
                2 | 3 => 6,
                _ => 36,
            },
            BlockType::Stick => 37,
            BlockType::Coal => 38,
            BlockType::IronIngot => 39,
            BlockType::GoldIngot => 40,
            BlockType::Diamond => 41,
            BlockType::WoodPickaxe => 42,
            BlockType::StonePickaxe => 43,
            BlockType::IronPickaxe => 44,
            BlockType::DiamondPickaxe => 45,
            BlockType::WoodAxe => 46,
            BlockType::StoneAxe => 47,
            BlockType::IronAxe => 48,
            BlockType::DiamondAxe => 49,
            BlockType::WoodShovel => 50,
            BlockType::StoneShovel => 51,
            BlockType::IronShovel => 52,
            BlockType::DiamondShovel => 53,
            BlockType::WoodSword => 54,
            BlockType::StoneSword => 55,
            BlockType::IronSword => 56,
            BlockType::DiamondSword => 57,
            BlockType::Apple => 58,
            BlockType::Bread => 59,
            BlockType::RawPorkchop => 60,
            BlockType::CookedPorkchop => 61,
            BlockType::RottenFlesh => 62,
        }
    }

    /// Base time in seconds required to break the block by hand in Survival mode
    #[inline(always)]
    pub fn break_time(self) -> f32 {
        match self {
            BlockType::Air | BlockType::Water | BlockType::Lava => f32::INFINITY,
            BlockType::Torch => 0.08,
            BlockType::Leaves => 0.25,
            BlockType::TNT => 0.25,
            BlockType::Glass | BlockType::Glowstone => 0.40,
            BlockType::Cactus | BlockType::Snow => 0.45,
            BlockType::Sand | BlockType::Gravel => 0.55,
            BlockType::Dirt | BlockType::Grass => 0.65,
            BlockType::Ice => 0.60,
            BlockType::Planks | BlockType::Bookshelf | BlockType::CraftingTable => 1.30,
            BlockType::Wood | BlockType::BirchWood | BlockType::SpruceWood => 1.60,
            BlockType::Cobblestone | BlockType::Brick | BlockType::Furnace => 2.20,
            BlockType::Stone => 2.00,
            BlockType::CoalOre => 2.40,
            BlockType::IronOre => 2.80,
            BlockType::GoldOre => 3.20,
            BlockType::DiamondOre => 3.80,
            BlockType::Obsidian => 8.00,
            BlockType::Bedrock => f32::INFINITY,
            _ => 1.0,
        }
    }

    /// What item/block is dropped when this block is destroyed
    #[inline(always)]
    pub fn drop_item(self) -> Option<BlockType> {
        match self {
            BlockType::Air | BlockType::Water | BlockType::Lava | BlockType::Bedrock => None,
            BlockType::Grass => Some(BlockType::Dirt),
            BlockType::Stone => Some(BlockType::Cobblestone),
            BlockType::CoalOre => Some(BlockType::Coal),
            BlockType::DiamondOre => Some(BlockType::Diamond),
            BlockType::Glass | BlockType::Ice => None,
            BlockType::Leaves => {
                // Leaves drop Leaves (or Apple handled in world break logic!)
                Some(BlockType::Leaves)
            }
            other => Some(other),
        }
    }
}
