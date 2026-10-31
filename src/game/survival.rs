use glam::Vec3;

pub struct SurvivalState {
    pub health: f32, // 0.0 to 20.0
    pub max_health: f32,
    pub hunger: f32, // 0.0 to 20.0
    pub max_hunger: f32,
    pub oxygen: f32, // 0.0 to 20.0
    pub max_oxygen: f32,
    pub highest_fall_y: f32,
    pub time_since_damage: f32,
    pub hurt_timer: f32, // Invulnerability & HUD damage flash timer
}

impl SurvivalState {
    pub fn new() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            hunger: 20.0,
            max_hunger: 20.0,
            oxygen: 20.0,
            max_oxygen: 20.0,
            highest_fall_y: 0.0,
            time_since_damage: 0.0,
            hurt_timer: 0.0,
        }
    }

    pub fn reset(&mut self, current_y: f32) {
        self.health = self.max_health;
        self.hunger = self.max_hunger;
        self.oxygen = self.max_oxygen;
        self.highest_fall_y = current_y;
        self.time_since_damage = 0.0;
        self.hurt_timer = 0.0;
    }

    pub fn take_damage(&mut self, amount: f32) -> bool {
        if amount <= 0.0 || self.hurt_timer > 0.15 {
            return false;
        }
        self.health = (self.health - amount).max(0.0);
        self.time_since_damage = 0.0;
        self.hurt_timer = 0.45;
        true
    }

    pub fn take_continuous_damage(&mut self, amount_per_sec: f32, dt: f32) {
        self.health = (self.health - amount_per_sec * dt).max(0.0);
        self.time_since_damage = 0.0;
        if self.hurt_timer <= 0.0 {
            self.hurt_timer = 0.25;
        }
    }

    pub fn eat_food(&mut self, restore: f32) -> bool {
        if self.hunger >= self.max_hunger - 0.1 && self.health >= self.max_health - 0.1 {
            return false;
        }
        self.hunger = (self.hunger + restore).min(self.max_hunger);
        self.health = (self.health + restore * 0.35).min(self.max_health);
        true
    }

    pub fn add_exhaustion(&mut self, amount: f32) {
        self.hunger = (self.hunger - amount).max(0.0);
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.001
    }

    pub fn update(
        &mut self,
        player_pos: Vec3,
        on_ground: bool,
        in_water: bool,
        head_underwater: bool,
        in_lava: bool,
        touching_cactus: bool,
        is_moving: bool,
        is_sprinting: bool,
        dt: f32,
    ) -> bool {
        self.time_since_damage += dt;
        if self.hurt_timer > 0.0 {
            self.hurt_timer = (self.hurt_timer - dt).max(0.0);
        }

        // Hunger drain from activity
        if is_moving {
            let drain_rate = if is_sprinting { 0.08 } else { 0.025 };
            self.hunger = (self.hunger - drain_rate * dt).max(0.0);
        }

        // Underwater oxygen bubbles & drowning
        if head_underwater {
            self.oxygen = (self.oxygen - 1.4 * dt).max(0.0);
            if self.oxygen <= 0.01 {
                self.take_continuous_damage(2.0, dt);
            }
        } else {
            self.oxygen = (self.oxygen + 8.0 * dt).min(self.max_oxygen);
        }

        // Environmental hazards
        if in_lava {
            self.take_continuous_damage(5.0, dt);
        }
        if touching_cactus {
            self.take_continuous_damage(1.8, dt);
        }

        // Void damage if falling below bedrock
        if player_pos.y < -12.0 {
            self.take_continuous_damage(12.0, dt);
        }

        // Natural health regeneration when well-fed (hunger >= 14), or starvation when hunger == 0
        if self.hunger >= 14.0 && self.time_since_damage > 3.5 && self.health < self.max_health && self.health > 0.0 {
            let heal = 0.65 * dt;
            self.health = (self.health + heal).min(self.max_health);
            self.hunger = (self.hunger - heal * 0.15).max(0.0);
        } else if self.hunger <= 0.01 && self.health > 2.0 {
            self.take_continuous_damage(0.35, dt);
        }

        // Water cushions all fall damage!
        if in_water {
            self.highest_fall_y = player_pos.y;
        } else if !on_ground {
            if player_pos.y > self.highest_fall_y {
                self.highest_fall_y = player_pos.y;
            }
        } else {
            // Just landed on solid ground
            if self.highest_fall_y > player_pos.y + 3.5 {
                let fall_dist = self.highest_fall_y - player_pos.y;
                let damage = (fall_dist - 3.0) * 1.8;
                self.take_damage(damage);
            }
            self.highest_fall_y = player_pos.y;
        }

        self.is_dead()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_survival_hunger_food_and_water_cushion() {
        let mut s = SurvivalState::new();
        s.reset(50.0);

        // Water cushions a 20-block fall completely
        s.highest_fall_y = 70.0;
        let dead = s.update(Vec3::new(0.0, 50.0, 0.0), true, true, false, false, false, false, false, 0.05);
        assert!(!dead);
        assert_eq!(s.health, 20.0);

        // Take damage and eat food to restore hunger & health
        assert!(s.take_damage(6.0));
        assert_eq!(s.health, 14.0);
        s.hunger = 12.0;
        assert!(s.eat_food(8.0));
        assert_eq!(s.hunger, 20.0);
        assert!(s.health > 14.0);
    }
}
