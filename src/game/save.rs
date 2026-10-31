use crate::game::inventory::{Inventory, ItemStack};
use crate::game::state::GameMode;
use crate::game::survival::SurvivalState;
use crate::voxel::block::BlockType;
use crate::voxel::world::World;
use glam::Vec3;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub const SAVE_FILE_PATH: &str = "rustcraft_save.dat";

pub struct WorldSaveData {
    pub seed: u32,
    pub game_mode: GameMode,
    pub time_of_day: f32,
    pub player_pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub hunger: f32,
    pub inventory: Inventory,
    pub modified_blocks: Vec<((i32, i32, i32), BlockType)>,
}

impl WorldSaveData {
    pub fn exists() -> bool {
        Path::new(SAVE_FILE_PATH).exists()
    }

    pub fn save(
        world: &World,
        game_mode: GameMode,
        time_of_day: f32,
        player_pos: Vec3,
        yaw: f32,
        pitch: f32,
        survival: &SurvivalState,
        inventory: &Inventory,
    ) -> std::io::Result<()> {
        let mut f = File::create(SAVE_FILE_PATH)?;
        let mode_id = if game_mode == GameMode::Creative { 1 } else { 0 };
        writeln!(f, "RUSTCRAFT_SAVE_V1")?;
        writeln!(
            f,
            "META {} {} {:.4} {:.3} {:.3} {:.3} {:.4} {:.4} {:.2} {:.2}",
            world.seed,
            mode_id,
            time_of_day,
            player_pos.x,
            player_pos.y,
            player_pos.z,
            yaw,
            pitch,
            survival.health,
            survival.hunger
        )?;

        for i in 0..36 {
            let stack = inventory.get_combined_slot(i);
            if !stack.is_empty() {
                writeln!(f, "SLOT {} {} {}", i, stack.block as u8, stack.count)?;
            }
        }

        for (&(x, y, z), &block) in &world.modified_blocks {
            writeln!(f, "BLK {} {} {} {}", x, y, z, block as u8)?;
        }

        Ok(())
    }

    pub fn load() -> Option<Self> {
        let file = File::open(SAVE_FILE_PATH).ok()?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let header = lines.next()?.ok()?;
        if header.trim() != "RUSTCRAFT_SAVE_V1" {
            return None;
        }

        let mut seed = 42;
        let mut game_mode = GameMode::Survival;
        let mut time_of_day = 0.22;
        let mut player_pos = Vec3::new(0.5, 45.0, 0.5);
        let mut yaw = -std::f32::consts::FRAC_PI_2;
        let mut pitch = 0.0;
        let mut health = 20.0;
        let mut hunger = 20.0;
        let mut inventory = Inventory::new_survival();
        let mut modified_blocks = Vec::new();

        for line_res in lines {
            let line = line_res.ok()?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            match parts[0] {
                "META" if parts.len() >= 11 => {
                    seed = parts[1].parse().unwrap_or(42);
                    let m: u8 = parts[2].parse().unwrap_or(0);
                    game_mode = if m == 1 { GameMode::Creative } else { GameMode::Survival };
                    time_of_day = parts[3].parse().unwrap_or(0.22);
                    let px = parts[4].parse().unwrap_or(0.5);
                    let py = parts[5].parse().unwrap_or(45.0);
                    let pz = parts[6].parse().unwrap_or(0.5);
                    player_pos = Vec3::new(px, py, pz);
                    yaw = parts[7].parse().unwrap_or(-std::f32::consts::FRAC_PI_2);
                    pitch = parts[8].parse().unwrap_or(0.0);
                    health = parts[9].parse().unwrap_or(20.0);
                    hunger = parts[10].parse().unwrap_or(20.0);
                }
                "SLOT" if parts.len() >= 4 => {
                    let idx: usize = parts[1].parse().unwrap_or(99);
                    let bid: u8 = parts[2].parse().unwrap_or(0);
                    let cnt: u32 = parts[3].parse().unwrap_or(0);
                    inventory.set_combined_slot(idx, ItemStack::new(BlockType::from_u8(bid), cnt));
                }
                "BLK" if parts.len() >= 5 => {
                    if let (Ok(x), Ok(y), Ok(z), Ok(bid)) = (
                        parts[1].parse::<i32>(),
                        parts[2].parse::<i32>(),
                        parts[3].parse::<i32>(),
                        parts[4].parse::<u8>(),
                    ) {
                        modified_blocks.push(((x, y, z), BlockType::from_u8(bid)));
                    }
                }
                _ => {}
            }
        }

        Some(Self {
            seed,
            game_mode,
            time_of_day,
            player_pos,
            yaw,
            pitch,
            health,
            hunger,
            inventory,
            modified_blocks,
        })
    }
}
