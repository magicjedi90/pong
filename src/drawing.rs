use engine_core::prelude::*;
use crate::achievements::DISPLAY_SECTIONS;
use crate::chaos_theme::ChaosTheme;
use crate::types::*;

impl PongGame {
    pub(crate) fn draw_ui(&self, ctx: &mut GameContext) {
        match &self.state {
            GameState::TitleScreen { selection } => self.draw_title(ctx, *selection),
            GameState::DifficultySelect { selection } => self.draw_difficulty(ctx, *selection),
            GameState::ChaosSelect { selection } => self.draw_chaos(ctx, *selection),
            GameState::Achievements => self.draw_achievements(ctx),
            _ => self.draw_gameplay(ctx),
        }
    }

    fn draw_title(&self, ctx: &mut GameContext, selection: u8) {
        let cx = ctx.window_size.x / 2.0;

        ctx.ui.label_centered("INSICULOUS PONG", Vec2::new(cx, 150.0));

        let items = ["Single Player", "Two Player", "Achievements"];
        for (i, item) in items.iter().enumerate() {
            let prefix = if i as u8 == selection { "> " } else { "  " };
            ctx.ui.label_centered(&format!("{prefix}{item}"), Vec2::new(cx, 240.0 + i as f32 * 30.0));
        }

        ctx.ui.label_centered("W/S or Arrows to navigate", Vec2::new(cx, 400.0));
        ctx.ui.label_centered("SPACE to confirm", Vec2::new(cx, 424.0));
    }

    fn draw_achievements(&self, ctx: &mut GameContext) {
        let cx = ctx.window_size.x / 2.0;
        let total = ctx.achievements.total();
        let unlocked = ctx.achievements.unlocked_count();

        ctx.ui.label_centered("ACHIEVEMENTS", Vec2::new(cx, 30.0));
        ctx.ui.label_centered(
            &format!("{unlocked} / {total} unlocked"),
            Vec2::new(cx, 54.0),
        );

        // Left-align the list. Pixel-perfect centering of variable-length rows
        // isn't worth the complexity — a fixed left margin reads fine.
        let left = 40.0;
        let mut y = 90.0;

        let locked_color = Color::new(0.45, 0.45, 0.5, 1.0);
        let unlocked_color = Color::new(1.0, 0.85, 0.25, 1.0);
        let desc_color = Color::new(0.75, 0.75, 0.8, 1.0);
        let header_color = Color::new(0.6, 0.75, 1.0, 1.0);

        for (section, ids) in DISPLAY_SECTIONS {
            ctx.ui.label_styled(section, Vec2::new(left, y), header_color, 16.0);
            y += 22.0;
            for id in *ids {
                let is_unlocked = ctx.achievements.is_unlocked(id);
                // Registry always has entries for these ids (registered in init).
                let Some(ach) = ctx.achievements.get(id) else { continue };

                let (marker, name_color) = if is_unlocked {
                    ("[X]", unlocked_color)
                } else {
                    ("[ ]", locked_color)
                };

                let (name, desc) = if !is_unlocked && ach.hidden {
                    ("???".to_string(), "Hidden — unlock to reveal".to_string())
                } else {
                    (ach.name.clone(), ach.description.clone())
                };

                ctx.ui.label_styled(
                    &format!("{marker} {name}"),
                    Vec2::new(left + 8.0, y),
                    name_color,
                    14.0,
                );
                ctx.ui.label_styled(&desc, Vec2::new(left + 52.0, y + 16.0), desc_color, 12.0);
                y += 36.0;
            }
            y += 6.0;
        }

        ctx.ui.label_centered("ESC or SPACE to go back", Vec2::new(cx, ctx.window_size.y - 20.0));
    }

    fn draw_difficulty(&self, ctx: &mut GameContext, selection: u8) {
        let cx = ctx.window_size.x / 2.0;

        ctx.ui.label_centered("SELECT DIFFICULTY", Vec2::new(cx, 150.0));

        let items = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
        for (i, diff) in items.iter().enumerate() {
            let prefix = if i as u8 == selection { "> " } else { "  " };
            ctx.ui.label_centered(&format!("{prefix}{}", diff.label()), Vec2::new(cx, 240.0 + i as f32 * 30.0));
        }

        ctx.ui.label_centered("SPACE to confirm, ESC to go back", Vec2::new(cx, 380.0));
    }

    fn draw_chaos(&self, ctx: &mut GameContext, selection: u8) {
        let cx = ctx.window_size.x / 2.0;

        ctx.ui.label_centered("SELECT CHAOS MODE", Vec2::new(cx, 130.0));

        for (i, mode) in ChaosMode::ALL.iter().enumerate() {
            let prefix = if i as u8 == selection { "> " } else { "  " };
            ctx.ui.label_centered(
                &format!("{prefix}{}", mode.label()),
                Vec2::new(cx, 200.0 + i as f32 * 30.0),
            );
        }

        let hint = match ChaosMode::ALL[selection as usize] {
            ChaosMode::Normal => "Classic Pong.",
            ChaosMode::Insane => "Ball doubles speed on every paddle hit.",
            ChaosMode::Ridiculous => "Match starts with two balls.",
            ChaosMode::Insiculous => "Two balls AND each doubles on paddle hits.",
        };
        ctx.ui.label_centered(hint, Vec2::new(cx, 360.0));
        ctx.ui.label_centered("SPACE to confirm, ESC to go back", Vec2::new(cx, 400.0));
    }

    fn draw_gameplay(&self, ctx: &mut GameContext) {
        let cx = ctx.window_size.x / 2.0;
        let cy = ctx.window_size.y / 2.0;

        let score_text = match self.mode {
            GameMode::SinglePlayer => format!("YOU {}  :  {} CPU", self.score_left, self.score_right),
            GameMode::TwoPlayer => format!("P1 {}  :  {} P2", self.score_left, self.score_right),
        };
        ctx.ui.label_centered(&score_text, Vec2::new(cx, 24.0));

        let theme = ChaosTheme::for_mode(self.chaos_mode);
        if let Some(banner) = theme.banner_text {
            let color = Color::new(theme.banner_color.x, theme.banner_color.y, theme.banner_color.z, theme.banner_color.w);
            ctx.ui.label_centered_styled(banner, Vec2::new(cx, ctx.window_size.y - 24.0), color, 16.0);
        }

        if self.speed_boost_timer > 0.0 {
            ctx.ui.label_centered(
                &format!("SPEED BOOST {:.1}s", self.speed_boost_timer),
                Vec2::new(cx, 48.0),
            );
        }

        match &self.state {
            GameState::Serving => {
                ctx.ui.label_centered("Press SPACE to serve", Vec2::new(cx, cy - 50.0));
                ctx.ui.label_centered("ESC for title screen", Vec2::new(cx, cy - 24.0));
            }
            GameState::GameOver { left_wins } => {
                let msg = match (self.mode, *left_wins) {
                    (GameMode::SinglePlayer, true) => "YOU WIN!",
                    (GameMode::SinglePlayer, false) => "CPU WINS!",
                    (GameMode::TwoPlayer, true) => "PLAYER 1 WINS!",
                    (GameMode::TwoPlayer, false) => "PLAYER 2 WINS!",
                };
                ctx.ui.label_centered(msg, Vec2::new(cx, cy - 60.0));
                ctx.ui.label_centered("SPACE to play again", Vec2::new(cx, cy - 34.0));
                ctx.ui.label_centered("ESC for title screen", Vec2::new(cx, cy - 8.0));
            }
            _ => {}
        }
    }
}
