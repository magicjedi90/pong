use engine_core::prelude::*;
use crate::chaos_theme::ChaosTheme;
use crate::types::*;

fn menu_navigate(current: u8, count: u8, up: bool, down: bool) -> u8 {
    if up {
        if current == 0 { count - 1 } else { current - 1 }
    } else if down {
        (current + 1) % count
    } else {
        current
    }
}

impl PongGame {
    pub(crate) fn update_title_input(&mut self, ctx: &mut GameContext, selection: u8) {
        let up = ctx.input.is_key_just_pressed(KeyCode::ArrowUp)
            || ctx.input.is_key_just_pressed(KeyCode::KeyW);
        let down = ctx.input.is_key_just_pressed(KeyCode::ArrowDown)
            || ctx.input.is_key_just_pressed(KeyCode::KeyS);
        let confirm = ctx.input.is_key_just_pressed(KeyCode::Space)
            || ctx.input.is_key_just_pressed(KeyCode::Enter);

        let selection = menu_navigate(selection, 2, up, down);
        self.state = GameState::TitleScreen { selection };

        if confirm {
            match selection {
                0 => self.state = GameState::DifficultySelect { selection: 1 },
                _ => {
                    self.mode = GameMode::TwoPlayer;
                    self.state = GameState::ChaosSelect { selection: 0 };
                }
            }
        }
    }

    pub(crate) fn update_difficulty_input(&mut self, ctx: &mut GameContext, selection: u8) {
        let up = ctx.input.is_key_just_pressed(KeyCode::ArrowUp)
            || ctx.input.is_key_just_pressed(KeyCode::KeyW);
        let down = ctx.input.is_key_just_pressed(KeyCode::ArrowDown)
            || ctx.input.is_key_just_pressed(KeyCode::KeyS);
        let confirm = ctx.input.is_key_just_pressed(KeyCode::Space)
            || ctx.input.is_key_just_pressed(KeyCode::Enter);
        let back = ctx.input.is_key_just_pressed(KeyCode::Escape);

        let selection = menu_navigate(selection, 3, up, down);
        self.state = GameState::DifficultySelect { selection };

        if back {
            self.state = GameState::TitleScreen { selection: 0 };
        } else if confirm {
            self.mode = GameMode::SinglePlayer;
            self.difficulty = match selection {
                0 => Difficulty::Easy,
                1 => Difficulty::Medium,
                _ => Difficulty::Hard,
            };
            self.state = GameState::ChaosSelect { selection: 0 };
        }
    }

    pub(crate) fn update_chaos_input(&mut self, ctx: &mut GameContext, selection: u8) {
        let up = ctx.input.is_key_just_pressed(KeyCode::ArrowUp)
            || ctx.input.is_key_just_pressed(KeyCode::KeyW);
        let down = ctx.input.is_key_just_pressed(KeyCode::ArrowDown)
            || ctx.input.is_key_just_pressed(KeyCode::KeyS);
        let confirm = ctx.input.is_key_just_pressed(KeyCode::Space)
            || ctx.input.is_key_just_pressed(KeyCode::Enter);
        let back = ctx.input.is_key_just_pressed(KeyCode::Escape);

        let count = ChaosMode::ALL.len() as u8;
        let selection = menu_navigate(selection, count, up, down);
        self.state = GameState::ChaosSelect { selection };

        if back {
            self.state = GameState::TitleScreen { selection: 0 };
        } else if confirm {
            self.chaos_mode = ChaosMode::ALL[selection as usize];
            self.start_game(&mut ctx.world);
        }
    }

    pub(crate) fn start_game(&mut self, world: &mut World) {
        self.score_left = 0;
        self.score_right = 0;
        self.last_scorer = Side::Right;
        self.extra_balls.clear();
        self.active_powerups.clear();
        self.speed_boost_timer = 0.0;
        self.powerup_spawn_timer = crate::constants::POWERUP_INITIAL_DELAY;
        self.ball_speed_mult.clear();
        self.apply_theme(world);
        self.reset_positions();
        self.state = GameState::Serving;
    }

    /// Push the current `chaos_mode`'s look onto the live entities: background
    /// tint, wall color, and the default ball color for any ball that hasn't
    /// been tinted by a paddle touch yet.
    pub(crate) fn apply_theme(&mut self, world: &mut World) {
        let theme = ChaosTheme::for_mode(self.chaos_mode);
        if let Some(bg) = self.background {
            if let Some(s) = world.get_mut::<Sprite>(bg) { s.color = theme.bg_color; }
        }
        for &w in &self.walls {
            if let Some(s) = world.get_mut::<Sprite>(w) { s.color = theme.wall_color; }
        }
        if let Some(ball) = self.ball {
            if let Some(s) = world.get_mut::<Sprite>(ball) { s.color = theme.ball_color; }
        }
        for &b in &self.extra_balls {
            if let Some(s) = world.get_mut::<Sprite>(b) { s.color = theme.ball_color; }
        }
    }
}
