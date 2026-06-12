//! Menu screens: navigation and selection. Match lifecycle (starting the
//! game, theming) lives in `gameplay::flow`.

use engine_core::prelude::*;
use crate::types::*;

/// One frame's worth of menu keys, read once per screen update.
struct MenuInput {
    up: bool,
    down: bool,
    confirm: bool,
    back: bool,
}

impl MenuInput {
    fn read(ctx: &GameContext) -> Self {
        Self {
            up: ctx.input.is_key_just_pressed(KeyCode::ArrowUp)
                || ctx.input.is_key_just_pressed(KeyCode::KeyW),
            down: ctx.input.is_key_just_pressed(KeyCode::ArrowDown)
                || ctx.input.is_key_just_pressed(KeyCode::KeyS),
            confirm: ctx.input.is_key_just_pressed(KeyCode::Space)
                || ctx.input.is_key_just_pressed(KeyCode::Enter),
            back: ctx.input.is_key_just_pressed(KeyCode::Escape),
        }
    }

    /// Move `current` through a `count`-item list with wraparound.
    fn navigate(&self, current: u8, count: u8) -> u8 {
        if self.up {
            if current == 0 { count - 1 } else { current - 1 }
        } else if self.down {
            (current + 1) % count
        } else {
            current
        }
    }
}

impl PongGame {
    pub(crate) fn update_title_input(&mut self, ctx: &mut GameContext, selection: u8) {
        let input = MenuInput::read(ctx);
        let selection = input.navigate(selection, 3);
        self.state = GameState::TitleScreen { selection };

        if input.confirm {
            match selection {
                0 => self.state = GameState::DifficultySelect { selection: 1 },
                1 => {
                    self.settings.mode = GameMode::TwoPlayer;
                    self.state = GameState::ChaosSelect { selection: 0 };
                }
                _ => self.state = GameState::Achievements,
            }
        }
    }

    pub(crate) fn update_achievements_input(&mut self, ctx: &mut GameContext) {
        let input = MenuInput::read(ctx);
        if input.back || input.confirm {
            self.state = GameState::TitleScreen { selection: 2 };
        }
    }

    pub(crate) fn update_difficulty_input(&mut self, ctx: &mut GameContext, selection: u8) {
        let input = MenuInput::read(ctx);
        let selection = input.navigate(selection, 3);
        self.state = GameState::DifficultySelect { selection };

        if input.back {
            self.state = GameState::TitleScreen { selection: 0 };
        } else if input.confirm {
            self.settings.mode = GameMode::SinglePlayer;
            self.settings.difficulty = match selection {
                0 => Difficulty::Easy,
                1 => Difficulty::Medium,
                _ => Difficulty::Hard,
            };
            self.state = GameState::ChaosSelect { selection: 0 };
        }
    }

    pub(crate) fn update_chaos_input(&mut self, ctx: &mut GameContext, selection: u8) {
        let input = MenuInput::read(ctx);
        let count = ChaosMode::ALL.len() as u8;
        let selection = input.navigate(selection, count);
        self.state = GameState::ChaosSelect { selection };

        if input.back {
            self.state = GameState::TitleScreen { selection: 0 };
        } else if input.confirm {
            self.settings.chaos = ChaosMode::ALL[selection as usize];
            // Mirror the runtime selection into the engine context so any
            // code reading ctx.chaos_mode agrees with self.settings.chaos.
            ctx.chaos_mode = self.settings.chaos;
            self.start_game(&mut ctx.world);
        }
    }
}
