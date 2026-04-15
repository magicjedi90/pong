//! Pong-specific achievement definitions and unlock logic.
//!
//! Registered once in `init()`; `unlock_win_achievements()` is called from the
//! game-over transition in `check_win_condition` whenever the player (left
//! paddle / player 1) wins.

use engine_core::prelude::*;

use crate::types::{Difficulty, GameMode, PongGame};

/// IDs — kept as `&'static str` so the compiler catches typos at call sites.
pub(crate) const BEAT_CPU_EASY:   &str = "beat_cpu_easy";
pub(crate) const BEAT_CPU_MEDIUM: &str = "beat_cpu_medium";
pub(crate) const BEAT_CPU_HARD:   &str = "beat_cpu_hard";

pub(crate) const WIN_NORMAL:      &str = "win_normal";
pub(crate) const WIN_INSANE:      &str = "win_insane";
pub(crate) const WIN_RIDICULOUS:  &str = "win_ridiculous";
pub(crate) const WIN_INSICULOUS:  &str = "win_insiculous";

pub(crate) const TWO_PLAYER:      &str = "two_player";

pub(crate) const SHUTOUT_NORMAL:     &str = "shutout_normal";
pub(crate) const SHUTOUT_INSANE:     &str = "shutout_insane";
pub(crate) const SHUTOUT_RIDICULOUS: &str = "shutout_ridiculous";
pub(crate) const SHUTOUT_INSICULOUS: &str = "shutout_insiculous";

/// Register every Pong achievement. Call once from `Game::init`.
pub(crate) fn register_all(mgr: &mut AchievementManager) {
    // Difficulty (CPU wins). Beating a higher difficulty cascades to unlock
    // easier ones too — handled at unlock time, not registration.
    mgr.register(Achievement::new(BEAT_CPU_EASY,
        "Training Wheels",
        "Beat the CPU on Easy."));
    mgr.register(Achievement::new(BEAT_CPU_MEDIUM,
        "Holding Your Own",
        "Beat the CPU on Medium."));
    mgr.register(Achievement::new(BEAT_CPU_HARD,
        "Pong Master",
        "Beat the CPU on Hard."));

    // Chaos mode wins
    mgr.register(Achievement::new(WIN_NORMAL,
        "First Victory",
        "Win a match in Normal mode."));
    mgr.register(Achievement::new(WIN_INSANE,
        "Insanely Good",
        "Win a match in Insane mode."));
    mgr.register(Achievement::new(WIN_RIDICULOUS,
        "This Is Ridiculous",
        "Win a match in Ridiculous mode."));
    mgr.register(Achievement::new(WIN_INSICULOUS,
        "Insiculous Champion",
        "Win a match in Insiculous mode."));

    // Multiplayer
    mgr.register(Achievement::new(TWO_PLAYER,
        "Friendly Rivalry",
        "Finish a 2-player match."));

    // Shutouts (difficulty ignored — any win where the opponent never scored)
    mgr.register(Achievement::new(SHUTOUT_NORMAL,
        "Clean Sheet",
        "Win a Normal-mode match without the opponent scoring."));
    mgr.register(Achievement::new(SHUTOUT_INSANE,
        "Untouchable",
        "Shut out the opponent in Insane mode."));
    mgr.register(Achievement::new(SHUTOUT_RIDICULOUS,
        "Impossibly Clean",
        "Shut out the opponent in Ridiculous mode."));
    mgr.register(Achievement::new(SHUTOUT_INSICULOUS,
        "Insiculously Dominant",
        "Shut out the opponent in Insiculous mode."));
}

impl PongGame {
    /// Called from `check_win_condition` when a match ends (either side won).
    /// The left paddle is always the local player (single-player) or player 1
    /// (two-player), so `left_wins` tells us whether the local player won.
    pub(crate) fn unlock_win_achievements(&self, ctx: &mut GameContext, left_wins: bool) {
        match self.mode {
            GameMode::TwoPlayer => {
                // "Friendly Rivalry" fires regardless of who won — it's for
                // *playing* a 2P match to completion.
                ctx.achievements.unlock(TWO_PLAYER);
            }
            GameMode::SinglePlayer if left_wins => {
                // CPU-win cascade: winning at a harder difficulty also grants
                // the easier ones (implies you could've won those too).
                let cpu_ids: &[&str] = match self.difficulty {
                    Difficulty::Easy   => &[BEAT_CPU_EASY],
                    Difficulty::Medium => &[BEAT_CPU_EASY, BEAT_CPU_MEDIUM],
                    Difficulty::Hard   => &[BEAT_CPU_EASY, BEAT_CPU_MEDIUM, BEAT_CPU_HARD],
                };
                for id in cpu_ids {
                    ctx.achievements.unlock(id);
                }

                // Chaos-mode win. Pong mutates `self.chaos_mode` from its own
                // menu, so it's the source of truth (not `ctx.chaos_mode`).
                ctx.achievements.unlock(chaos_win_id(self.chaos_mode));

                // Shutout — mode-specific, difficulty ignored.
                if self.score_right == 0 {
                    ctx.achievements.unlock(chaos_shutout_id(self.chaos_mode));
                }
            }
            _ => {} // Single-player loss — no achievements.
        }
    }
}

fn chaos_win_id(mode: ChaosMode) -> &'static str {
    match mode {
        ChaosMode::Normal     => WIN_NORMAL,
        ChaosMode::Insane     => WIN_INSANE,
        ChaosMode::Ridiculous => WIN_RIDICULOUS,
        ChaosMode::Insiculous => WIN_INSICULOUS,
    }
}

fn chaos_shutout_id(mode: ChaosMode) -> &'static str {
    match mode {
        ChaosMode::Normal     => SHUTOUT_NORMAL,
        ChaosMode::Insane     => SHUTOUT_INSANE,
        ChaosMode::Ridiculous => SHUTOUT_RIDICULOUS,
        ChaosMode::Insiculous => SHUTOUT_INSICULOUS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_all_adds_twelve() {
        let mut mgr = AchievementManager::in_memory();
        register_all(&mut mgr);
        assert_eq!(mgr.total(), 12);
    }

    #[test]
    fn chaos_win_id_maps_each_mode() {
        assert_eq!(chaos_win_id(ChaosMode::Normal),     WIN_NORMAL);
        assert_eq!(chaos_win_id(ChaosMode::Insane),     WIN_INSANE);
        assert_eq!(chaos_win_id(ChaosMode::Ridiculous), WIN_RIDICULOUS);
        assert_eq!(chaos_win_id(ChaosMode::Insiculous), WIN_INSICULOUS);
    }

    #[test]
    fn chaos_shutout_id_maps_each_mode() {
        assert_eq!(chaos_shutout_id(ChaosMode::Normal),     SHUTOUT_NORMAL);
        assert_eq!(chaos_shutout_id(ChaosMode::Insane),     SHUTOUT_INSANE);
        assert_eq!(chaos_shutout_id(ChaosMode::Ridiculous), SHUTOUT_RIDICULOUS);
        assert_eq!(chaos_shutout_id(ChaosMode::Insiculous), SHUTOUT_INSICULOUS);
    }

    #[test]
    fn every_id_is_registered() {
        let mut mgr = AchievementManager::in_memory();
        register_all(&mut mgr);
        for id in [
            BEAT_CPU_EASY, BEAT_CPU_MEDIUM, BEAT_CPU_HARD,
            WIN_NORMAL, WIN_INSANE, WIN_RIDICULOUS, WIN_INSICULOUS,
            TWO_PLAYER,
            SHUTOUT_NORMAL, SHUTOUT_INSANE, SHUTOUT_RIDICULOUS, SHUTOUT_INSICULOUS,
        ] {
            assert!(mgr.get(id).is_some(), "{} not registered", id);
        }
    }
}
