use glam::Vec3;

pub struct SurvivalState {
    pub health: f32, // 0.0 to 20.0
    pub max_health: f32,
    pub highest_fall_y: f32,
    pub time_since_damage: f32,
}

impl SurvivalState {
    pub fn new() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            highest_fall_y: 0.0,
            time_since_damage: 0.0,
        }
    }

    pub fn reset(&mut self, current_y: f32) {
        self.health = self.max_health;
        self.highest_fall_y = current_y;
        self.time_since_damage = 0.0;
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        self.time_since_damage = 0.0;
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.001
    }

    pub fn update(&mut self, player_pos: Vec3, on_ground: bool, dt: f32) -> bool {
        self.time_since_damage += dt;

        // Slow natural regeneration if above half health and no recent damage
        if self.time_since_damage > 4.0 && self.health < self.max_health && self.health > 0.0 {
            self.health = (self.health + 0.5 * dt).min(self.max_health);
        }

        if !on_ground {
            if player_pos.y > self.highest_fall_y {
                self.highest_fall_y = player_pos.y;
            }
        } else {
            // Just landed on ground
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
