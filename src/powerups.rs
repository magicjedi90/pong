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
        if self.active_powerups.len() >= MAX_POWERUPS {
            return;
        }

        self.powerup_spawn_timer -= ctx.delta_time;
        if self.powerup_spawn_timer > 0.0 {
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

        let color = match kind {
            PowerUpKind::SpeedBoost => SPEED_BOOST_COLOR,
            PowerUpKind::MultiBall => MULTIBALL_COLOR,
        };

        let entity = ctx.world.spawn()
            .with(Transform2D::from_parts(Vec2::new(x, y), 0.0, Vec2::splat(POWERUP_SCALE)))
            .with(Sprite::new(self.tex_id).with_color(color))
            .with(RigidBody::new_static())
            .with(Collider::circle_collider(POWERUP_SIZE / 2.0).as_sensor())
            .id();

        self.active_powerups.push(SpawnedPowerUp { entity, kind });

        // Reset timer to random interval
        let t = hash_f32(self.frame_count.wrapping_add(3));
        self.powerup_spawn_timer = POWERUP_SPAWN_MIN + t * (POWERUP_SPAWN_MAX - POWERUP_SPAWN_MIN);
    }

    pub(crate) fn check_powerup_collisions(&mut self, ctx: &mut GameContext) {
        let all_balls: Vec<EntityId> = self.ball.into_iter()
            .chain(self.extra_balls.iter().copied())
            .collect();

        // Collect what was hit (index, kind, which ball triggered it)
        let mut hits: Vec<(usize, PowerUpKind, EntityId)> = Vec::new();
        for (i, powerup) in self.active_powerups.iter().enumerate() {
            for collision in self.physics.collision_events() {
                if !collision.event.started { continue; }
                for &b in &all_balls {
                    if collision.event.involves(b, powerup.entity) {
                        hits.push((i, powerup.kind, b));
                    }
                }
            }
        }

        // Apply effects and remove consumed power-ups
        let mut consumed_indices: Vec<usize> = hits.iter().map(|(i, _, _)| *i).collect();
        consumed_indices.dedup();

        for &(_, kind, ball_id) in &hits {
            match kind {
                PowerUpKind::SpeedBoost => {
                    self.speed_boost_timer = SPEED_BOOST_DURATION;
                }
                PowerUpKind::MultiBall => {
                    self.spawn_extra_ball(ctx, ball_id);
                }
            }
        }

        for &i in consumed_indices.iter().rev() {
            let powerup = self.active_powerups.remove(i);
            self.physics.destroy_entity(&mut ctx.world, powerup.entity);
        }
    }

    fn spawn_extra_ball(&mut self, ctx: &mut GameContext, source_ball: EntityId) {
        // Get source ball position and X sign (for "opposite direction" flavor)
        let pos = ctx.world.get::<Transform2D>(source_ball)
            .map(|t| t.position)
            .unwrap_or(Vec2::ZERO);
        let source_vx = self.physics.get_body_velocity(source_ball)
            .map(|(v, _)| v.x)
            .unwrap_or(BALL_INITIAL_SPEED);

        // Spawn new ball at source position
        let entity = self.spawn_ball(&mut ctx.world, self.tex_id);
        if let Some(transform) = ctx.world.get_mut::<Transform2D>(entity) {
            transform.position = pos;
        }

        // Use the same direction formula as the initial serve so multi-ball
        // spawns have real vertical spread instead of a shallow mirror of the
        // source ball's trajectory. Fires opposite the source for gameplay.
        let dir_x = if source_vx >= 0.0 { -1.0 } else { 1.0 };
        let hash = self.frame_count.wrapping_mul(2654435761).wrapping_add(0xB5297A4D);
        let t = ((hash >> 16) as f32) / 65535.0;
        let dir_y = t * 1.2 - 0.6;
        let dir = Vec2::new(dir_x, dir_y).normalize();
        self.physics.set_velocity(entity, dir * BALL_INITIAL_SPEED, 0.0);

        // Match source ball color
        if let Some(source_sprite) = ctx.world.get::<Sprite>(source_ball) {
            let color = source_sprite.color;
            if let Some(sprite) = ctx.world.get_mut::<Sprite>(entity) {
                sprite.color = color;
            }
        }

        self.extra_balls.push(entity);
    }

    pub(crate) fn update_speed_boost(&mut self, delta_time: f32) {
        if self.speed_boost_timer > 0.0 {
            self.speed_boost_timer = (self.speed_boost_timer - delta_time).max(0.0);
        }
    }

    pub(crate) fn destroy_extra_ball(&mut self, world: &mut World, ball: EntityId) {
        self.physics.destroy_entity(world, ball);
        self.extra_balls.retain(|&b| b != ball);
    }

    pub(crate) fn destroy_all_extra_balls(&mut self, world: &mut World) {
        for ball in self.extra_balls.drain(..) {
            self.physics.destroy_entity(world, ball);
        }
    }

    pub(crate) fn destroy_all_powerups(&mut self, world: &mut World) {
        for powerup in self.active_powerups.drain(..) {
            self.physics.destroy_entity(world, powerup.entity);
        }
    }
}
