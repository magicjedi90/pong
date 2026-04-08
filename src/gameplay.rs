use engine_core::prelude::*;
use crate::constants::*;
use crate::types::*;

fn entity_y(world: &World, entity: EntityId) -> f32 {
    world.get::<Transform2D>(entity).map(|t| t.position.y).unwrap_or(0.0)
}

impl PongGame {
    pub(crate) fn update_gameplay(&mut self, ctx: &mut GameContext) {
        let ball = match self.ball { Some(e) => e, None => return };
        let left_paddle = match self.left_paddle { Some(e) => e, None => return };
        let right_paddle = match self.right_paddle { Some(e) => e, None => return };

        self.update_left_paddle(ctx, left_paddle);
        self.update_right_paddle(ctx, right_paddle, ball);
        self.physics.update(&mut ctx.world, ctx.delta_time);
        self.flush_pending_launches();

        self.handle_gameplay_input(ctx, ball);
        self.maintain_all_ball_velocities();
        self.check_goals(ctx);
        self.check_powerup_collisions(ctx);
        self.update_powerup_spawns(ctx);
        self.update_speed_boost(ctx.delta_time);
        self.check_win_condition(&mut ctx.world);
    }

    fn update_left_paddle(&mut self, ctx: &GameContext, paddle: EntityId) {
        let (up, down) = match self.mode {
            GameMode::SinglePlayer => (
                ctx.input.is_key_pressed(KeyCode::KeyW) || ctx.input.is_key_pressed(KeyCode::ArrowUp),
                ctx.input.is_key_pressed(KeyCode::KeyS) || ctx.input.is_key_pressed(KeyCode::ArrowDown),
            ),
            GameMode::TwoPlayer => (
                ctx.input.is_key_pressed(KeyCode::KeyW),
                ctx.input.is_key_pressed(KeyCode::KeyS),
            ),
        };
        let dy = match (up, down) {
            (true, false) => PADDLE_SPEED,
            (false, true) => -PADDLE_SPEED,
            _ => 0.0,
        };
        let y = entity_y(&ctx.world, paddle);
        let new_y = (y + dy * ctx.delta_time).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);
        self.physics.set_kinematic_target(paddle, Vec2::new(-PADDLE_X, new_y), 0.0);
    }

    fn update_right_paddle(&mut self, ctx: &GameContext, paddle: EntityId, ball: EntityId) {
        match self.mode {
            GameMode::SinglePlayer => {
                let ball_y = entity_y(&ctx.world, ball);
                let paddle_y = entity_y(&ctx.world, paddle);
                let diff = ball_y - paddle_y;
                let speed = self.difficulty.ai_speed();
                let dead_zone = self.difficulty.ai_dead_zone();
                let dy = if diff.abs() > dead_zone { diff.signum() * speed } else { 0.0 };
                let new_y = (paddle_y + dy * ctx.delta_time).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);
                self.physics.set_kinematic_target(paddle, Vec2::new(PADDLE_X, new_y), 0.0);
            }
            GameMode::TwoPlayer => {
                let up = ctx.input.is_key_pressed(KeyCode::ArrowUp);
                let down = ctx.input.is_key_pressed(KeyCode::ArrowDown);
                let dy = match (up, down) {
                    (true, false) => PADDLE_SPEED,
                    (false, true) => -PADDLE_SPEED,
                    _ => 0.0,
                };
                let y = entity_y(&ctx.world, paddle);
                let new_y = (y + dy * ctx.delta_time).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);
                self.physics.set_kinematic_target(paddle, Vec2::new(PADDLE_X, new_y), 0.0);
            }
        }
    }

    fn handle_gameplay_input(&mut self, ctx: &mut GameContext, ball: EntityId) {
        match &self.state {
            GameState::Serving => {
                if ctx.input.is_key_just_pressed(KeyCode::Escape) {
                    self.reset_to_title(&mut ctx.world);
                } else if ctx.input.is_key_just_pressed(KeyCode::Space) {
                    let dir_x = match self.last_scorer {
                        Side::Left => -1.0,
                        Side::Right => 1.0,
                    };
                    let hash = self.frame_count.wrapping_mul(2654435761);
                    let t = ((hash >> 16) as f32) / 65535.0;
                    let dir_y = t * 1.2 - 0.6;
                    let dir = Vec2::new(dir_x, dir_y).normalize();
                    self.physics.apply_impulse(ball, dir * BALL_INITIAL_SPEED);
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver { .. } => {
                if ctx.input.is_key_just_pressed(KeyCode::Space) {
                    self.start_game();
                } else if ctx.input.is_key_just_pressed(KeyCode::Escape) {
                    self.reset_to_title(&mut ctx.world);
                }
            }
            _ => {}
        }
    }

    fn maintain_all_ball_velocities(&mut self) {
        let all_balls: Vec<EntityId> = self.ball.into_iter()
            .chain(self.extra_balls.iter().copied())
            .collect();
        for ball in all_balls {
            self.maintain_ball_velocity(ball);
        }
    }

    fn maintain_ball_velocity(&mut self, ball: EntityId) {
        let Some((vel, _)) = self.physics.get_body_velocity(ball) else { return };
        if vel.x.abs() < 0.1 { return; }

        let speed = if self.speed_boost_timer > 0.0 {
            BALL_INITIAL_SPEED * SPEED_BOOST_MULTIPLIER
        } else {
            BALL_INITIAL_SPEED
        };
        let max_vy = if self.speed_boost_timer > 0.0 {
            BALL_MAX_SPEED * SPEED_BOOST_MULTIPLIER
        } else {
            BALL_MAX_SPEED
        };

        let fixed_vx = vel.x.signum() * speed;
        let vy = vel.y.clamp(-max_vy, max_vy);
        let new_vel = Vec2::new(fixed_vx, vy);

        if (new_vel - vel).length() > 1.0 {
            self.physics.set_body_velocity(ball, new_vel, 0.0);
        }
    }

    fn check_goals(&mut self, ctx: &mut GameContext) {
        let left_goal = match self.left_goal { Some(e) => e, None => return };
        let right_goal = match self.right_goal { Some(e) => e, None => return };
        let left_paddle = match self.left_paddle { Some(e) => e, None => return };
        let right_paddle = match self.right_paddle { Some(e) => e, None => return };

        let all_balls: Vec<EntityId> = self.ball.into_iter()
            .chain(self.extra_balls.iter().copied())
            .collect();

        // Collect scoring events and paddle touches
        let mut balls_scored: Vec<(EntityId, Side)> = Vec::new();
        for collision in self.physics.collision_events() {
            if !collision.event.started { continue; }
            for &b in &all_balls {
                if collision.event.involves(b, left_paddle) {
                    self.last_touch = Some(Side::Left);
                    if let Some(sprite) = ctx.world.get_mut::<Sprite>(b) {
                        sprite.color = LEFT_COLOR;
                    }
                } else if collision.event.involves(b, right_paddle) {
                    self.last_touch = Some(Side::Right);
                    if let Some(sprite) = ctx.world.get_mut::<Sprite>(b) {
                        sprite.color = RIGHT_COLOR;
                    }
                }
                if collision.event.involves(b, left_goal) {
                    balls_scored.push((b, Side::Right));
                } else if collision.event.involves(b, right_goal) {
                    balls_scored.push((b, Side::Left));
                }
            }
        }

        // Process scored balls
        for (ball_id, side) in &balls_scored {
            match side {
                Side::Left => self.score_left += 1,
                Side::Right => self.score_right += 1,
            }
            self.last_scorer = *side;

            if Some(*ball_id) == self.ball {
                // Primary ball scored — promote an extra if available
                if let Some(promoted) = self.extra_balls.pop() {
                    self.ball = Some(promoted);
                } else {
                    self.ball = None;
                }
                // Destroy the old primary
                ctx.world.remove_entity(ball_id).ok();
                self.physics.physics_world_mut().remove_entity(*ball_id);
            } else {
                // Extra ball scored — just destroy it
                self.destroy_extra_ball(&mut ctx.world, *ball_id);
            }
        }

        // If no balls remain, reset to serving
        if self.ball.is_none() && self.extra_balls.is_empty() && !balls_scored.is_empty() {
            self.destroy_all_powerups(&mut ctx.world);
            self.speed_boost_timer = 0.0;
            self.last_touch = None;
            // Spawn a fresh primary ball
            self.ball = Some(self.spawn_ball(&mut ctx.world, self.tex_id));
            self.physics.reset_body(self.ball.unwrap(), Vec2::ZERO);
            self.state = GameState::Serving;
        }
    }

    fn check_win_condition(&mut self, world: &mut World) {
        if !matches!(self.state, GameState::Playing | GameState::Serving) { return; }

        let winner = if self.score_left >= WIN_SCORE {
            Some(true)
        } else if self.score_right >= WIN_SCORE {
            Some(false)
        } else {
            None
        };

        if let Some(left_wins) = winner {
            self.destroy_all_extra_balls(world);
            self.destroy_all_powerups(world);
            self.speed_boost_timer = 0.0;
            self.state = GameState::GameOver { left_wins };
        }
    }

    fn reset_to_title(&mut self, world: &mut World) {
        self.destroy_all_extra_balls(world);
        self.destroy_all_powerups(world);
        self.speed_boost_timer = 0.0;
        self.reset_positions();
        self.state = GameState::TitleScreen { selection: 0 };
    }

    pub(crate) fn update_entity_visibility(&self, ctx: &mut GameContext) {
        let alpha = match self.state {
            GameState::TitleScreen { .. } | GameState::DifficultySelect { .. } => 0.0,
            _ => 1.0,
        };
        let entities = [self.ball, self.left_paddle, self.right_paddle].into_iter().flatten()
            .chain(self.extra_balls.iter().copied())
            .chain(self.active_powerups.iter().map(|p| p.entity));
        for entity in entities {
            if let Some(sprite) = ctx.world.get_mut::<Sprite>(entity) {
                sprite.color.w = alpha;
            }
        }
    }

    pub(crate) fn reset_positions(&mut self) {
        if let Some(ball) = self.ball {
            self.physics.reset_body(ball, Vec2::ZERO);
        }
        if let Some(lp) = self.left_paddle {
            self.physics.set_kinematic_target(lp, Vec2::new(-PADDLE_X, 0.0), 0.0);
        }
        if let Some(rp) = self.right_paddle {
            self.physics.set_kinematic_target(rp, Vec2::new(PADDLE_X, 0.0), 0.0);
        }
    }
}
