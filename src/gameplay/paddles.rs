//! Paddle movement: keyboard control for players, ball-tracking AI for the CPU.

use engine_core::prelude::*;
use crate::constants::*;
use crate::types::*;
use super::entity_y;

/// Paddle speed from a pair of up/down inputs.
fn paddle_dy(up: bool, down: bool) -> f32 {
    match (up, down) {
        (true, false) => PADDLE_SPEED,
        (false, true) => -PADDLE_SPEED,
        _ => 0.0,
    }
}

impl PongGame {
    pub(crate) fn update_paddles(&mut self, ctx: &GameContext) {
        let (Some(left), Some(right)) = (self.playfield.left_paddle, self.playfield.right_paddle)
        else { return };
        self.update_left_paddle(ctx, left);
        self.update_right_paddle(ctx, right);
    }

    fn update_left_paddle(&mut self, ctx: &GameContext, paddle: EntityId) {
        // In single player the lone human gets both key sets; in two player
        // W/S belongs to player 1 and the arrows to player 2.
        let (up, down) = match self.settings.mode {
            GameMode::SinglePlayer => (
                ctx.input.is_key_pressed(KeyCode::KeyW) || ctx.input.is_key_pressed(KeyCode::ArrowUp),
                ctx.input.is_key_pressed(KeyCode::KeyS) || ctx.input.is_key_pressed(KeyCode::ArrowDown),
            ),
            GameMode::TwoPlayer => (
                ctx.input.is_key_pressed(KeyCode::KeyW),
                ctx.input.is_key_pressed(KeyCode::KeyS),
            ),
        };
        self.move_paddle(ctx, paddle, -PADDLE_X, paddle_dy(up, down));
    }

    fn update_right_paddle(&mut self, ctx: &GameContext, paddle: EntityId) {
        let dy = match self.settings.mode {
            GameMode::SinglePlayer => self.ai_dy(ctx, paddle),
            GameMode::TwoPlayer => paddle_dy(
                ctx.input.is_key_pressed(KeyCode::ArrowUp),
                ctx.input.is_key_pressed(KeyCode::ArrowDown),
            ),
        };
        self.move_paddle(ctx, paddle, PADDLE_X, dy);
    }

    /// CPU control: chase the primary ball's Y at the difficulty's speed,
    /// with a dead zone so easier CPUs wobble less precisely.
    fn ai_dy(&self, ctx: &GameContext, paddle: EntityId) -> f32 {
        let Some(ball) = self.balls.primary else { return 0.0 };
        let diff = entity_y(&ctx.world, ball) - entity_y(&ctx.world, paddle);
        if diff.abs() > self.settings.difficulty.ai_dead_zone() {
            diff.signum() * self.settings.difficulty.ai_speed()
        } else {
            0.0
        }
    }

    fn move_paddle(&mut self, ctx: &GameContext, paddle: EntityId, x: f32, dy: f32) {
        let y = entity_y(&ctx.world, paddle);
        let new_y = (y + dy * ctx.delta_time).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);
        self.physics.set_kinematic_target(paddle, Vec2::new(x, new_y), 0.0);
    }
}
