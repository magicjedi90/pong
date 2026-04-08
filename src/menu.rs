use engine_core::prelude::*;
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
                    self.start_game();
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
            self.start_game();
        }
    }

    pub(crate) fn start_game(&mut self) {
        self.score_left = 0;
        self.score_right = 0;
        self.last_scorer = Side::Right;
        self.extra_balls.clear();
        self.active_powerups.clear();
        self.speed_boost_timer = 0.0;
        self.powerup_spawn_timer = crate::constants::POWERUP_INITIAL_DELAY;
        self.reset_positions();
        self.state = GameState::Serving;
    }
}
