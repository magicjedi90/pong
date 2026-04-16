mod achievements;
mod chaos_theme;
mod constants;
mod drawing;
mod gameplay;
mod menu;
mod power_ups;
mod spawning;
mod types;

use engine_core::prelude::*;
use chaos_theme::ChaosTheme;
use constants::*;
use spawning::*;
use types::*;

impl Game for PongGame {
    fn init(&mut self, ctx: &mut GameContext) {
        if let Ok(font) = ctx.ui.load_font_file("assets/fonts/font.ttf") {
            ctx.ui.set_default_font(font);
        }

        achievements::register_all(ctx.achievements);

        let tex = ctx.assets.create_solid_color(1, 1, [255, 255, 255, 255]).unwrap();
        self.tex_id = tex.id;

        let theme = ChaosTheme::for_mode(self.chaos_mode);
        self.background = Some(spawn_background(&mut ctx.world, tex.id, theme.bg_color));

        self.left_paddle = Some(spawn_paddle(&mut ctx.world, -PADDLE_X, tex.id, LEFT_COLOR));
        self.right_paddle = Some(spawn_paddle(&mut ctx.world, PADDLE_X, tex.id, RIGHT_COLOR));
        self.ball = Some(self.spawn_ball(&mut ctx.world, tex.id));

        let wall_y = WIN_H / 2.0 - 10.0;
        self.walls.push(spawn_wall(&mut ctx.world, Vec2::new(0.0, wall_y), WIN_W, 20.0, tex.id, theme.wall_color));
        self.walls.push(spawn_wall(&mut ctx.world, Vec2::new(0.0, -wall_y), WIN_W, 20.0, tex.id, theme.wall_color));

        let goal_x = WIN_W / 2.0 + 10.0;
        self.left_goal = Some(spawn_goal_sensor(&mut ctx.world, -goal_x));
        self.right_goal = Some(spawn_goal_sensor(&mut ctx.world, goal_x));
    }

    fn update(&mut self, ctx: &mut GameContext) {
        self.frame_count = self.frame_count.wrapping_add(1);

        match self.state.clone() {
            GameState::TitleScreen { selection } => self.update_title_input(ctx, selection),
            GameState::DifficultySelect { selection } => self.update_difficulty_input(ctx, selection),
            GameState::ChaosSelect { selection } => self.update_chaos_input(ctx, selection),
            GameState::Achievements => self.update_achievements_input(ctx),
            _ => self.update_gameplay(ctx),
        }

        self.update_entity_visibility(ctx);
        self.draw_ui(ctx);
    }
}

fn main() {
    let config = GameConfig::new("Insiculous Pong")
        .with_size(WIN_W as u32, WIN_H as u32)
        .with_clear_color(0.0, 0.0, 0.0, 1.0)
        .with_fps(60)
        .with_achievement_save_path("saves/pong_achievements.json");

    run_game(PongGame::default(), config).unwrap();
}
