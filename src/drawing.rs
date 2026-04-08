use engine_core::prelude::*;
use crate::types::*;

impl PongGame {
    pub(crate) fn draw_ui(&self, ctx: &mut GameContext) {
        match &self.state {
            GameState::TitleScreen { selection } => self.draw_title(ctx, *selection),
            GameState::DifficultySelect { selection } => self.draw_difficulty(ctx, *selection),
            _ => self.draw_gameplay(ctx),
        }
    }

    fn draw_title(&self, ctx: &mut GameContext, selection: u8) {
        let cx = ctx.window_size.x / 2.0;

        ctx.ui.label_centered("INSICULOUS PONG", Vec2::new(cx, 150.0));

        let items = ["Single Player", "Two Player"];
        for (i, item) in items.iter().enumerate() {
            let prefix = if i as u8 == selection { "> " } else { "  " };
            ctx.ui.label_centered(&format!("{prefix}{item}"), Vec2::new(cx, 240.0 + i as f32 * 30.0));
        }

        ctx.ui.label_centered("W/S or Arrows to navigate", Vec2::new(cx, 380.0));
        ctx.ui.label_centered("SPACE to confirm", Vec2::new(cx, 404.0));
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

    fn draw_gameplay(&self, ctx: &mut GameContext) {
        let cx = ctx.window_size.x / 2.0;
        let cy = ctx.window_size.y / 2.0;

        let score_text = match self.mode {
            GameMode::SinglePlayer => format!("YOU {}  :  {} CPU", self.score_left, self.score_right),
            GameMode::TwoPlayer => format!("P1 {}  :  {} P2", self.score_left, self.score_right),
        };
        ctx.ui.label_centered(&score_text, Vec2::new(cx, 24.0));

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
