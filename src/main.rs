mod achievements;
mod chaos_theme;
mod constants;
mod drawing;
mod effects;
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

/// Directory that holds the game's `assets/` and `saves/` folders.
///
/// Prefers the executable's directory (shipped layout: assets next to the
/// binary), falling back to the crate directory so `cargo run` works from
/// any current working directory.
fn game_root() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("assets").is_dir() {
                return dir.to_path_buf();
            }
        }
    }
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

impl Game for PongGame {
    fn init(&mut self, ctx: &mut GameContext) {
        let font_path = game_root().join("assets/fonts/font.ttf");
        if let Ok(font) = ctx.ui.load_font_file(&font_path.to_string_lossy()) {
            ctx.ui.set_default_font(font);
        }

        achievements::register_all(ctx.achievements);

        let tex = ctx.assets.create_solid_color(1, 1, [255, 255, 255, 255]).unwrap();
        self.tex_id = tex.id;
        // Relative paths resolve against the asset base path set in main().
        self.paddle_tex_id = ctx.assets.load_texture("paddle_16px.png")
            .expect("missing assets/paddle_16px.png").id;
        self.ball_tex_id = ctx.assets.load_texture("ball_8px.png")
            .expect("missing assets/ball_8px.png").id;

        let theme = ChaosTheme::for_mode(self.chaos_mode);
        self.background = Some(spawn_background(&mut ctx.world, tex.id, theme.bg_color));

        // Left paddle: rounded face naturally on the right (toward the ball).
        // Right paddle: mirror so its rounded face points left (toward the ball).
        self.left_paddle = Some(spawn_paddle(&mut ctx.world, -PADDLE_X, self.paddle_tex_id, LEFT_COLOR, false));
        self.right_paddle = Some(spawn_paddle(&mut ctx.world, PADDLE_X, self.paddle_tex_id, RIGHT_COLOR, true));
        self.ball = Some(self.spawn_ball(&mut ctx.world));

        let wall_y = WIN_H / 2.0 - 10.0;
        self.walls.push(spawn_wall(&mut ctx.world, Vec2::new(0.0, wall_y), WIN_W, 20.0, tex.id, theme.wall_color));
        self.walls.push(spawn_wall(&mut ctx.world, Vec2::new(0.0, -wall_y), WIN_W, 20.0, tex.id, theme.wall_color));

        let goal_x = WIN_W / 2.0 + 10.0;
        self.left_goal = Some(spawn_goal_sensor(&mut ctx.world, -goal_x));
        self.right_goal = Some(spawn_goal_sensor(&mut ctx.world, goal_x));

        // Build the deforming grid background.
        self.grid = Some(effects::build_grid(&theme));
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
    // Anchor assets and saves to the game's directory so launching from any
    // working directory behaves the same.
    let root = game_root();
    let config = GameConfig::new("Insiculous Pong")
        .with_size(WIN_W as u32, WIN_H as u32)
        .with_clear_color(0.0, 0.0, 0.0, 1.0)
        .with_fps(60)
        .with_asset_base_path(root.join("assets").to_string_lossy())
        .with_achievement_save_path(root.join("saves/pong_achievements.json").to_string_lossy());

    // With `--features editor` the game runs inside the scene editor
    // (hierarchy, inspector, gizmos, play/pause/stop, collider overlay);
    // without it the game runs bare. Same game code either way.
    #[cfg(feature = "editor")]
    editor_integration::run_game_with_editor(PongGame::default(), config).unwrap();
    #[cfg(not(feature = "editor"))]
    run_game(PongGame::default(), config).unwrap();
}
