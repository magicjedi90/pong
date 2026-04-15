use engine_core::prelude::*;

/// Per-mode presentation tokens: background tint, wall/ball accent colors,
/// and an optional HUD banner shown during gameplay.
///
/// Gameplay branches live in `gameplay.rs` and read `ChaosMode` directly.
/// This module owns only the *look* of each mode so art tweaks stay in one
/// place.
pub(crate) struct ChaosTheme {
    pub bg_color: Vec4,
    pub wall_color: Vec4,
    pub ball_color: Vec4,
    pub banner_text: Option<&'static str>,
    pub banner_color: Vec4,
}

impl ChaosTheme {
    pub(crate) fn for_mode(mode: ChaosMode) -> Self {
        match mode {
            ChaosMode::Normal => Self {
                bg_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                wall_color: Vec4::new(0.35, 0.35, 0.42, 1.0),
                ball_color: Vec4::ONE,
                banner_text: None,
                banner_color: Vec4::ONE,
            },
            ChaosMode::Insane => Self {
                bg_color: Vec4::new(0.18, 0.02, 0.02, 1.0),
                wall_color: Vec4::new(1.0, 0.4, 0.2, 1.0),
                ball_color: Vec4::new(1.0, 0.82, 0.6, 1.0),
                banner_text: Some("!! INSANE !!"),
                banner_color: Vec4::new(1.0, 0.5, 0.3, 1.0),
            },
            ChaosMode::Ridiculous => Self {
                bg_color: Vec4::new(0.08, 0.02, 0.15, 1.0),
                wall_color: Vec4::new(0.9, 0.3, 1.0, 1.0),
                ball_color: Vec4::new(1.0, 0.75, 1.0, 1.0),
                banner_text: Some("~~ RIDICULOUS ~~"),
                banner_color: Vec4::new(0.95, 0.4, 1.0, 1.0),
            },
            ChaosMode::Insiculous => Self {
                bg_color: Vec4::new(0.04, 0.08, 0.04, 1.0),
                wall_color: Vec4::new(0.5, 1.0, 0.3, 1.0),
                ball_color: Vec4::new(0.85, 1.0, 0.55, 1.0),
                banner_text: Some(">>> INSICULOUS <<<"),
                banner_color: Vec4::new(0.7, 1.0, 0.4, 1.0),
            },
        }
    }
}
