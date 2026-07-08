//! Power-up timing and pickup effects. Entity creation lives in
//! `spawning.rs`; the multi-ball spawn itself lives in `gameplay::balls`.

use engine_core::prelude::*;
use crate::constants::*;
use crate::types::*;

/// Simple hash for pseudo-random values from frame count.
fn hash(seed: u32) -> u32 {
    seed.wrapping_mul(2654435761)
}

/// Map a hash to a float in [0, 1).
fn hash_f32(seed: u32) -> f32 {
    (hash(seed) >> 8) as f32 / 16777216.0
}

impl PongGame {
    pub(crate) fn update_powerup_spawns(&mut self, ctx: &mut GameContext) {
        if !matches!(self.state, GameState::Playing) {
            return;
        }
        if self.power_ups.active.len() >= MAX_POWERUPS {
            return;
        }

        self.power_ups.spawn_timer -= ctx.delta_time;
        if self.power_ups.spawn_timer > 0.0 {
            return;
        }

        // Pick random kind
        let kind = if hash(self.frame_count).is_multiple_of(2) {
            PowerUpKind::SpeedBoost
        } else {
            PowerUpKind::MultiBall
        };

        // Random position in the middle area (avoid paddles and edges)
        let x = hash_f32(self.frame_count.wrapping_add(1)) * 400.0 - 200.0;
        let y = hash_f32(self.frame_count.wrapping_add(2)) * 400.0 - 200.0;
        self.spawn_power_up(ctx.world, kind, Vec2::new(x, y));

        // Reset timer to random interval
        let t = hash_f32(self.frame_count.wrapping_add(3));
        self.power_ups.spawn_timer = POWERUP_SPAWN_MIN + t * (POWERUP_SPAWN_MAX - POWERUP_SPAWN_MIN);
    }

    pub(crate) fn check_powerup_collisions(
        &mut self,
        ctx: &mut GameContext,
        collisions: &[CollisionData],
    ) {
        let all_balls = self.balls.all();

        // Engine-side collection: each pickup grants its effect exactly once,
        // even if two balls touch it in the same frame.
        let collected =
            self.power_ups
                .active
                .collect(collisions, &all_balls, &mut self.physics, ctx.world);

        for (kind, ball_id) in collected {
            match kind {
                PowerUpKind::SpeedBoost => {
                    self.power_ups.speed_boost.start(SPEED_BOOST_DURATION);
                }
                PowerUpKind::MultiBall => {
                    self.spawn_extra_ball(ctx, ball_id);
                }
            }
        }
    }

    pub(crate) fn update_speed_boost(&mut self, delta_time: f32) {
        // Speed boost has no visuals to revert, so the expiry signal is unused.
        let _ = self.power_ups.speed_boost.tick(delta_time);
    }

    pub(crate) fn destroy_all_powerups(&mut self, world: &mut World) {
        self.power_ups.active.clear(&mut self.physics, world);
    }
}
