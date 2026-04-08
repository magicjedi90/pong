use engine_core::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Side { Left, Right }

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum GameMode { SinglePlayer, TwoPlayer }

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Difficulty { Easy, Medium, Hard }

impl Difficulty {
    pub(crate) fn ai_speed(self) -> f32 {
        match self {
            Difficulty::Easy => 180.0,
            Difficulty::Medium => 255.0,
            Difficulty::Hard => 380.0,
        }
    }

    pub(crate) fn ai_dead_zone(self) -> f32 {
        match self {
            Difficulty::Easy => 15.0,
            Difficulty::Medium => 2.0,
            Difficulty::Hard => 0.5,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum GameState {
    TitleScreen { selection: u8 },
    DifficultySelect { selection: u8 },
    Serving,
    Playing,
    GameOver { left_wins: bool },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PowerUpKind { SpeedBoost, MultiBall }

pub(crate) struct SpawnedPowerUp {
    pub(crate) entity: EntityId,
    pub(crate) kind: PowerUpKind,
}

pub(crate) struct PongGame {
    pub(crate) physics: PhysicsSystem,

    pub(crate) left_paddle: Option<EntityId>,
    pub(crate) right_paddle: Option<EntityId>,
    pub(crate) ball: Option<EntityId>,
    pub(crate) extra_balls: Vec<EntityId>,
    pub(crate) left_goal: Option<EntityId>,
    pub(crate) right_goal: Option<EntityId>,
    pub(crate) tex_id: u32,

    pub(crate) score_left: u32,
    pub(crate) score_right: u32,
    pub(crate) last_scorer: Side,
    pub(crate) state: GameState,
    pub(crate) mode: GameMode,
    pub(crate) difficulty: Difficulty,
    pub(crate) last_touch: Option<Side>,
    pub(crate) frame_count: u32,

    // Power-ups
    pub(crate) active_powerups: Vec<SpawnedPowerUp>,
    pub(crate) speed_boost_timer: f32,
    pub(crate) powerup_spawn_timer: f32,
    pub(crate) pending_launches: Vec<(EntityId, Vec2)>,
}

impl Default for PongGame {
    fn default() -> Self {
        Self {
            physics: PhysicsSystem::with_config(PhysicsConfig::top_down()),
            left_paddle: None,
            right_paddle: None,
            ball: None,
            extra_balls: Vec::new(),
            left_goal: None,
            right_goal: None,
            tex_id: 0,
            score_left: 0,
            score_right: 0,
            last_scorer: Side::Right,
            last_touch: None,
            state: GameState::TitleScreen { selection: 0 },
            mode: GameMode::SinglePlayer,
            difficulty: Difficulty::Medium,
            frame_count: 0,
            active_powerups: Vec::new(),
            speed_boost_timer: 0.0,
            powerup_spawn_timer: crate::constants::POWERUP_INITIAL_DELAY,
            pending_launches: Vec::new(),
        }
    }
}
