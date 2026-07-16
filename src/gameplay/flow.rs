//! Match lifecycle: serving, game-over input, starting/leaving a match,
//! position resets, and hiding gameplay entities while menus are up.

use engine_core::prelude::*;
use crate::constants::*;
use crate::types::*;
use super::balls::serve_direction;

impl PongGame {
    /// State transitions during a match: serve from `Serving`, restart or
    /// bail to the title screen from `GameOver`. Either player's primary
    /// action (Space/Enter/pad A) or menu action (Escape/pad Start) counts.
    pub(crate) fn handle_gameplay_input(&mut self, ctx: &mut GameContext) {
        let primary = ctx.players.just_activated_any(GameAction::Action1, ctx.input);
        let menu = ctx.players.just_activated_any(GameAction::Menu, ctx.input);
        match &self.state {
            GameState::Serving => {
                if menu {
                    self.reset_to_title(ctx.world);
                } else if primary {
                    self.serve(ctx);
                }
            }
            GameState::GameOver { .. } => {
                if primary {
                    self.start_game(ctx.world);
                } else if menu {
                    self.reset_to_title(ctx.world);
                }
            }
            _ => {}
        }
    }

    /// Launch the primary ball toward the last scorer's opponent. In
    /// Ridiculous mode a second ball heads the opposite way so each player
    /// gets one incoming.
    fn serve(&mut self, ctx: &mut GameContext) {
        let Some(ball) = self.balls.primary else { return };
        let dir_x = match self.score.last_scorer {
            Side::Left => -1.0,
            Side::Right => 1.0,
        };
        let dir = serve_direction(self.frame_count, 0, dir_x);
        self.physics.set_velocity(ball, dir * BALL_INITIAL_SPEED, 0.0);

        if self.settings.chaos.is_ridiculous() {
            let name = self.next_extra_ball_name();
            let extra = self.spawn_ball(ctx.world, &name);
            let ball_color = self.current_theme().accent_color;
            if let Some(s) = ctx.world.get_mut::<Sprite>(extra) {
                s.color = ball_color;
            }
            let dir2 = serve_direction(self.frame_count, 0x9E37, -dir_x);
            self.physics.set_velocity(extra, dir2 * BALL_INITIAL_SPEED, 0.0);
            self.balls.extras.push(extra);
        }

        self.state = GameState::Playing;
    }

    /// Begin a fresh match with the current settings: zero the score, drop
    /// all transient state, re-theme the playfield, and wait for the serve.
    pub(crate) fn start_game(&mut self, world: &mut World) {
        self.score.reset();
        self.destroy_all_extra_balls(world);
        self.destroy_all_powerups(world);
        self.power_ups = PowerUpState::default();
        self.balls.speed_mult.clear();
        self.apply_theme(world);
        self.reset_positions();
        self.state = GameState::Serving;
    }

    pub(crate) fn reset_to_title(&mut self, world: &mut World) {
        self.destroy_all_extra_balls(world);
        self.destroy_all_powerups(world);
        self.power_ups.speed_boost.stop();
        self.reset_positions();
        self.state = GameState::TitleScreen { selection: 0 };
    }

    /// Push the current chaos mode's look onto the live entities: background
    /// tint, wall color, and the default ball color for any ball that hasn't
    /// been tinted by a paddle touch yet.
    pub(crate) fn apply_theme(&mut self, world: &mut World) {
        let theme = self.current_theme();
        if let Some(bg) = self.playfield.background {
            if let Some(s) = world.get_mut::<Sprite>(bg) { s.color = theme.bg_color; }
        }
        for &w in &self.playfield.walls {
            if let Some(s) = world.get_mut::<Sprite>(w) { s.color = theme.structure_color; }
        }
        for ball in self.balls.all() {
            if let Some(s) = world.get_mut::<Sprite>(ball) { s.color = theme.accent_color; }
        }
    }

    pub(crate) fn reset_positions(&mut self) {
        if let Some(ball) = self.balls.primary {
            self.physics.reset_body(ball, Vec2::ZERO);
        }
        if let Some(lp) = self.playfield.left_paddle {
            self.physics.set_kinematic_target(lp, Vec2::new(-PADDLE_X, 0.0), 0.0);
        }
        if let Some(rp) = self.playfield.right_paddle {
            self.physics.set_kinematic_target(rp, Vec2::new(PADDLE_X, 0.0), 0.0);
        }
    }

    /// Gameplay sprites only render during a match — menus get a bare screen.
    pub(crate) fn update_entity_visibility(&self, ctx: &mut GameContext) {
        let visible = !matches!(
            self.state,
            GameState::TitleScreen { .. }
                | GameState::DifficultySelect { .. }
                | GameState::ChaosSelect { .. }
                | GameState::Achievements
        );
        let entities = [self.playfield.left_paddle, self.playfield.right_paddle].into_iter().flatten()
            .chain(self.balls.all())
            .chain(self.playfield.walls.iter().copied())
            .chain(self.power_ups.active.entities());
        set_sprites_visible(ctx.world, entities, visible);
    }
}
