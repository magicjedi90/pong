//! Paddle movement: player control via the engine's per-player input
//! bindings (keyboard, dpad, or analog stick), ball-tracking AI for the CPU.

use engine_core::prelude::*;
use crate::constants::*;
use crate::types::*;
use super::entity_y;

/// Paddle speed from a merged movement axis in `-1.0..=1.0` (+1 = up).
/// Analog input scales the speed; overdriven input is clamped.
fn paddle_dy(axis: f32) -> f32 {
    axis.clamp(-1.0, 1.0) * PADDLE_SPEED
}

impl PongGame {
    pub(crate) fn update_paddles(&mut self, ctx: &GameContext) {
        let (Some(left), Some(right)) = (self.playfield.left_paddle, self.playfield.right_paddle)
        else { return };
        self.update_left_paddle(ctx, left);
        self.update_right_paddle(ctx, right);
    }

    fn update_left_paddle(&mut self, ctx: &GameContext, paddle: EntityId) {
        // In single player the lone human gets both players' devices (WASD,
        // arrows, and either pad); in two player the left paddle is P1's.
        let axis = match self.settings.mode {
            GameMode::SinglePlayer => {
                ctx.players.move_y(PlayerId::P1, ctx.input)
                    + ctx.players.move_y(PlayerId::P2, ctx.input)
            }
            GameMode::TwoPlayer => ctx.players.move_y(PlayerId::P1, ctx.input),
        };
        self.move_paddle(ctx, paddle, -PADDLE_X, paddle_dy(axis));
    }

    fn update_right_paddle(&mut self, ctx: &GameContext, paddle: EntityId) {
        let dy = match self.settings.mode {
            GameMode::SinglePlayer => self.ai_dy(ctx, paddle),
            GameMode::TwoPlayer => paddle_dy(ctx.players.move_y(PlayerId::P2, ctx.input)),
        };
        self.move_paddle(ctx, paddle, PADDLE_X, dy);
    }

    /// CPU control: chase the primary ball's Y at the difficulty's speed,
    /// with a dead zone so easier CPUs wobble less precisely.
    fn ai_dy(&self, ctx: &GameContext, paddle: EntityId) -> f32 {
        let Some(ball) = self.balls.primary else { return 0.0 };
        let diff = entity_y(ctx.world, ball) - entity_y(ctx.world, paddle);
        if diff.abs() > self.settings.difficulty.ai_dead_zone() {
            diff.signum() * self.settings.difficulty.ai_speed()
        } else {
            0.0
        }
    }

    fn move_paddle(&mut self, ctx: &GameContext, paddle: EntityId, x: f32, dy: f32) {
        let y = entity_y(ctx.world, paddle);
        let new_y = (y + dy * ctx.delta_time).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);
        self.physics.set_kinematic_target(paddle, Vec2::new(x, new_y), 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paddle_dy_scales_linearly_with_axis() {
        assert_eq!(paddle_dy(1.0), PADDLE_SPEED);
        assert_eq!(paddle_dy(-1.0), -PADDLE_SPEED);
        assert_eq!(paddle_dy(0.5), PADDLE_SPEED * 0.5);
    }

    #[test]
    fn paddle_dy_clamps_overdriven_axis() {
        // Merged key + stick input can sum past 1.0 before clamping
        assert_eq!(paddle_dy(1.8), PADDLE_SPEED);
        assert_eq!(paddle_dy(-1.8), -PADDLE_SPEED);
    }

    #[test]
    fn paddle_dy_zero_axis_holds_still() {
        assert_eq!(paddle_dy(0.0), 0.0);
    }
}
