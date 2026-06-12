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
        let kind = if hash(self.frame_count) % 2 == 0 {
            PowerUpKind::SpeedBoost
        } else {
            PowerUpKind::MultiBall
        };

        // Random position in the middle area (avoid paddles and edges)
        let x = hash_f32(self.frame_count.wrapping_add(1)) * 400.0 - 200.0;
        let y = hash_f32(self.frame_count.wrapping_add(2)) * 400.0 - 200.0;
        self.spawn_power_up(&mut ctx.world, kind, Vec2::new(x, y));

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

        // Collect what was hit (index, kind, which ball triggered it)
        let mut hits: Vec<(usize, PowerUpKind, EntityId)> = Vec::new();
        for collision in collisions {
            if !collision.event.started { continue; }
            for (i, powerup) in self.power_ups.active.iter().enumerate() {
                for &b in &all_balls {
                    if collision.event.involves(b, powerup.entity) {
                        hits.push((i, powerup.kind, b));
                    }
                }
            }
        }

        // Apply effects and remove consumed power-ups. Sort before dedup —
        // hits are ordered by collision event, so equal indices may not be
        // adjacent, and the reverse removal below needs ascending order.
        let mut consumed_indices: Vec<usize> = hits.iter().map(|(i, _, _)| *i).collect();
        consumed_indices.sort_unstable();
        consumed_indices.dedup();

        for &(_, kind, ball_id) in &hits {
            match kind {
                PowerUpKind::SpeedBoost => {
                    self.power_ups.speed_boost_timer = SPEED_BOOST_DURATION;
                }
                PowerUpKind::MultiBall => {
                    self.spawn_extra_ball(ctx, ball_id);
                }
            }
        }

        for &i in consumed_indices.iter().rev() {
            let powerup = self.power_ups.active.remove(i);
            self.physics.destroy_entity(&mut ctx.world, powerup.entity);
        }
    }

    pub(crate) fn update_speed_boost(&mut self, delta_time: f32) {
        if self.power_ups.speed_boost_timer > 0.0 {
            self.power_ups.speed_boost_timer =
                (self.power_ups.speed_boost_timer - delta_time).max(0.0);
        }
    }

    pub(crate) fn destroy_all_powerups(&mut self, world: &mut World) {
        for powerup in self.power_ups.active.drain(..) {
            self.physics.destroy_entity(world, powerup.entity);
        }
    }
}
