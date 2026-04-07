use engine_core::prelude::*;
use crate::types::*;

impl PongGame {
    pub(crate) fn draw_ui(&self, ctx: &mut GameContext) {
        ctx.ui.begin_frame(ctx.input, ctx.window_size);

        match &self.state {
            GameState::TitleScreen { selection } => self.draw_title(ctx, *selection),
            GameState::DifficultySelect { selection } => self.draw_difficulty(ctx, *selection),
            _ => self.draw_gameplay(ctx),
        }

        ctx.ui.end_frame();
    }

    fn draw_title(&self, ctx: &mut GameContext, selection: u8) {
        let cx = ctx.window_size.x / 2.0;

        ctx.ui.label("INSICULOUS PONG", Vec2::new(cx - 68.0, 150.0));

        let items = ["Single Player", "Two Player"];
        for (i, item) in items.iter().enumerate() {
            let prefix = if i as u8 == selection { "> " } else { "  " };
            ctx.ui.label(&format!("{prefix}{item}"), Vec2::new(cx - 68.0, 240.0 + i as f32 * 30.0));
        }

        ctx.ui.label("W/S or Arrows to navigate", Vec2::new(cx - 108.0, 380.0));
        ctx.ui.label("SPACE to confirm", Vec2::new(cx - 72.0, 404.0));
    }

    fn draw_difficulty(&self, ctx: &mut GameContext, selection: u8) {
        let cx = ctx.window_size.x / 2.0;

        ctx.ui.label("SELECT DIFFICULTY", Vec2::new(cx - 76.0, 150.0));

        let items = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
        for (i, diff) in items.iter().enumerate() {
            let prefix = if i as u8 == selection { "> " } else { "  " };
            ctx.ui.label(&format!("{prefix}{}", diff.label()), Vec2::new(cx - 48.0, 240.0 + i as f32 * 30.0));
        }

        ctx.ui.label("SPACE to confirm, ESC to go back", Vec2::new(cx - 140.0, 380.0));
    }

    fn draw_gameplay(&self, ctx: &mut GameContext) {
        let cx = ctx.window_size.x / 2.0;
        let cy = ctx.window_size.y / 2.0;

        let score_text = match self.mode {
            GameMode::SinglePlayer => format!("YOU {}  :  {} CPU", self.score_left, self.score_right),
            GameMode::TwoPlayer => format!("P1 {}  :  {} P2", self.score_left, self.score_right),
        };
        ctx.ui.label(&score_text, Vec2::new(cx - 64.0, 24.0));

        match &self.state {
            GameState::Serving => {
                ctx.ui.label("Press SPACE to serve", Vec2::new(cx - 88.0, cy - 20.0));
                ctx.ui.label("ESC for title screen", Vec2::new(cx - 88.0, cy + 6.0));
            }
            GameState::GameOver { left_wins } => {
                let msg = match (self.mode, *left_wins) {
                    (GameMode::SinglePlayer, true) => "YOU WIN!",
                    (GameMode::SinglePlayer, false) => "CPU WINS!",
                    (GameMode::TwoPlayer, true) => "PLAYER 1 WINS!",
                    (GameMode::TwoPlayer, false) => "PLAYER 2 WINS!",
                };
                ctx.ui.label(msg, Vec2::new(cx - 60.0, cy - 30.0));
                ctx.ui.label("SPACE to play again", Vec2::new(cx - 84.0, cy));
                ctx.ui.label("ESC for title screen", Vec2::new(cx - 88.0, cy + 26.0));
            }
            _ => {}
        }
    }
}
