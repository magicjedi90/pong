//! All on-screen text: menu screens, the achievements page, and the in-match
//! HUD (score, banners, serve/game-over prompts). Menu screens draw inside
//! the engine's `MenuPanel` window chrome, styled from the chaos theme.

use engine_core::prelude::*;
use crate::achievements::DISPLAY_SECTIONS;
use crate::types::*;

impl PongGame {
    fn menu_style(&self) -> MenuStyle {
        MenuStyle::from_theme(&self.current_theme())
    }

    pub(crate) fn draw_ui(&self, ctx: &mut GameContext) {
        match &self.state {
            GameState::TitleScreen { selection } => self.draw_title(ctx, *selection),
            GameState::DifficultySelect { selection } => self.draw_difficulty(ctx, *selection),
            GameState::ChaosSelect { selection } => self.draw_chaos(ctx, *selection),
            GameState::Achievements => self.draw_achievements(ctx),
            _ => self.draw_gameplay_hud(ctx),
        }
    }

    fn draw_title(&self, ctx: &mut GameContext, selection: u8) {
        let style = self.menu_style();
        let panel = MenuPanel::new("INSICULOUS PONG", ctx.window_size / 2.0, 360.0, 4);
        let mut y = panel.begin(ctx.ui, &style);
        let items = ["Single Player", "Two Player", "Achievements", "Exit"];
        for (i, item) in items.iter().enumerate() {
            y = panel.item(ctx.ui, y, item, i as u8 == selection, &style);
        }
        panel.hint(ctx.ui, "W/S or D-Pad navigate - SPACE or (A) confirm", &style);
    }

    fn draw_achievements(&self, ctx: &mut GameContext) {
        let style = self.menu_style();
        let total = ctx.achievements.total();
        let unlocked = ctx.achievements.unlocked_count();

        // Tall window covering most of the screen; the section list draws
        // left-aligned inside its bounds.
        let panel = MenuPanel::new("ACHIEVEMENTS", ctx.window_size / 2.0, ctx.window_size.x - 120.0, 15);
        let first_y = panel.begin(ctx.ui, &style);
        let rect = panel.panel_rect();

        ctx.ui.label_centered(
            &format!("{unlocked} / {total} unlocked"),
            Vec2::new(ctx.window_size.x / 2.0, first_y - 8.0),
        );

        let left = rect.x + 28.0;
        let mut y = first_y + 18.0;

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

        panel.hint(ctx.ui, "ESC or SPACE to go back", &style);
    }

    fn draw_difficulty(&self, ctx: &mut GameContext, selection: u8) {
        let style = self.menu_style();
        let panel = MenuPanel::new("SELECT DIFFICULTY", ctx.window_size / 2.0, 360.0, 3);
        let mut y = panel.begin(ctx.ui, &style);
        let items = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
        for (i, diff) in items.iter().enumerate() {
            y = panel.item(ctx.ui, y, diff.label(), i as u8 == selection, &style);
        }
        panel.hint(ctx.ui, "SPACE to confirm - ESC to go back", &style);
    }

    fn draw_chaos(&self, ctx: &mut GameContext, selection: u8) {
        let style = self.menu_style();
        let panel = MenuPanel::new("SELECT CHAOS MODE", ctx.window_size / 2.0, 400.0, 4);
        let mut y = panel.begin(ctx.ui, &style);
        for (i, mode) in ChaosMode::ALL.iter().enumerate() {
            y = panel.item(ctx.ui, y, mode.label(), i as u8 == selection, &style);
        }

        let hint = match ChaosMode::ALL[selection as usize] {
            ChaosMode::Normal => "Classic Pong.",
            ChaosMode::Insane => "Ball doubles speed on every paddle hit.",
            ChaosMode::Ridiculous => "Match starts with two balls.",
            ChaosMode::Insiculous => "Two balls AND each doubles on paddle hits.",
        };
        panel.hint(ctx.ui, hint, &style);
    }

    fn draw_gameplay_hud(&self, ctx: &mut GameContext) {
        let cx = ctx.window_size.x / 2.0;
        let cy = ctx.window_size.y / 2.0;

        let score_text = match self.settings.mode {
            GameMode::SinglePlayer => format!("YOU {}  :  {} CPU", self.score.left, self.score.right),
            GameMode::TwoPlayer => format!("P1 {}  :  {} P2", self.score.left, self.score.right),
        };
        ctx.ui.label_centered(&score_text, Vec2::new(cx, 24.0));

        let theme = self.current_theme();
        if let Some(banner) = theme.banner_text {
            let color = Color::new(theme.banner_color.x, theme.banner_color.y, theme.banner_color.z, theme.banner_color.w);
            ctx.ui.label_centered_styled(banner, Vec2::new(cx, ctx.window_size.y - 24.0), color, 16.0);
        }

        if self.power_ups.speed_boost.active() {
            ctx.ui.label_centered(
                &format!("SPEED BOOST {:.1}s", self.power_ups.speed_boost.remaining()),
                Vec2::new(cx, 48.0),
            );
        }

        match &self.state {
            GameState::Serving => {
                ctx.ui.label_centered("Press SPACE to serve", Vec2::new(cx, cy - 50.0));
                ctx.ui.label_centered("ESC to pause", Vec2::new(cx, cy - 24.0));
            }
            GameState::GameOver { left_wins } => {
                let msg = match (self.settings.mode, *left_wins) {
                    (GameMode::SinglePlayer, true) => "YOU WIN!",
                    (GameMode::SinglePlayer, false) => "CPU WINS!",
                    (GameMode::TwoPlayer, true) => "PLAYER 1 WINS!",
                    (GameMode::TwoPlayer, false) => "PLAYER 2 WINS!",
                };
                let style = self.menu_style();
                let panel = MenuPanel::new(msg, Vec2::new(cx, cy), 340.0, 1);
                let y = panel.begin(ctx.ui, &style);
                panel.line(ctx.ui, y, "SPACE to play again", &style);
                panel.hint(ctx.ui, "ESC for title screen", &style);
            }
            _ => {}
        }

        if self.pause.is_active() {
            let style = self.menu_style();
            self.pause.draw(ctx.ui, ctx.window_size, &style);
        }
    }
}
