//! The in-match update loop, split by responsibility:
//!
//! - [`paddles`] — player input and CPU AI paddle movement
//! - [`balls`] — ball velocity maintenance, extra-ball spawning/teardown
//! - [`scoring`] — goal detection, point awards, win condition
//! - [`flow`] — match lifecycle (serve, game over, reset, entity visibility)

mod balls;
mod flow;
mod paddles;
mod scoring;

use engine_core::prelude::*;
use crate::types::*;

pub(crate) fn entity_position(world: &World, entity: EntityId) -> Option<Vec2> {
    world.get::<Transform2D>(entity).map(|t| t.position)
}

pub(crate) fn entity_y(world: &World, entity: EntityId) -> f32 {
    world.get::<Transform2D>(entity).map(|t| t.position.y).unwrap_or(0.0)
}

impl PongGame {
    pub(crate) fn update_gameplay(&mut self, ctx: &mut GameContext) {
        if self.balls.primary.is_none()
            || self.playfield.left_paddle.is_none()
            || self.playfield.right_paddle.is_none()
        {
            return;
        }

        // F1 toggles the collider debug overlay. Magenta outlines render on
        // top of sprites so any sprite-vs-collider mismatch is obvious.
        if ctx.input.is_key_just_pressed(KeyCode::F1) {
            self.debug_colliders = !self.debug_colliders;
        }

        self.update_paddles(ctx);
        self.physics.update(&mut ctx.world, ctx.delta_time);

        // Snapshot this frame's collision events once. Every consumer below
        // works from this slice instead of re-reading collision_events(), so
        // gameplay never depends on how long physics retains its buffer.
        let collisions: Vec<CollisionData> = self.physics.collision_events().to_vec();

        self.handle_gameplay_input(ctx);
        self.maintain_all_ball_velocities();
        self.check_goals(ctx, &collisions);
        self.check_powerup_collisions(ctx, &collisions);
        self.update_powerup_spawns(ctx);
        self.update_speed_boost(ctx.delta_time);
        self.check_win_condition(ctx);

        // Step + render the deforming grid after gameplay so it reacts to
        // this frame's collisions.
        self.step_and_emit_grid(ctx);
    }

    /// Advance the spring-mass grid and push its line vertices into the
    /// engine's per-frame line buffer. When the collider-debug overlay is
    /// enabled, the collider outlines are pushed on top.
    fn step_and_emit_grid(&mut self, ctx: &mut GameContext) {
        if let Some(grid) = self.grid.as_mut() {
            grid.step(ctx.delta_time);
            let verts = grid.build_line_vertices();
            ctx.lines.extend_from_slice(verts);
        }
        if self.debug_colliders {
            // Bright magenta with high emissive so the outline blooms and
            // sits visibly above every sprite.
            debug::draw_colliders(&ctx.world, ctx.lines, Vec4::new(1.0, 0.2, 1.0, 0.9), 2.0);
        }
    }

    /// Push a radial shockwave into the deforming grid (paddle hits, goals).
    pub(crate) fn ripple_grid(&mut self, position: Vec2, strength: f32, radius: f32) {
        if let Some(grid) = self.grid.as_mut() {
            grid.apply_impulse(&GridImpulse::Radial {
                position,
                strength,
                radius,
                attractive: false,
            });
        }
    }
}
