//! Visual effect presets — particle configs and grid setup.
//!
//! Centralizes the look of each event (paddle hit, goal explosion) so tuning
//! happens in one place.

use engine_core::prelude::*;

use crate::chaos_theme::ChaosTheme;

/// 32×24-node grid sized to cover the playfield with some overscan.
pub(crate) fn build_grid(theme: &ChaosTheme) -> GridMesh {
    GridMesh::new(32, 24, 36.0, Vec2::ZERO)
        .with_color(theme.grid_color)
        .with_emissive(0.7)
        .with_stiffness(60.0)
        .with_damping(0.07)
}

/// Particles thrown off a paddle hit. Cone aligned with the bounce normal so
/// the spray reads as kinetic.
pub(crate) fn paddle_hit_burst(color: Vec4, normal: Vec2, theme: &ChaosTheme, tex: u32) -> ParticleConfig {
    let count = (24.0 * theme.particle_count_mult).round() as usize;
    ParticleConfig::burst(count)
        .with_lifetime(0.25, 0.55)
        .with_speed(140.0, 320.0)
        .with_direction(normal, std::f32::consts::FRAC_PI_3) // ~60° half-cone
        .with_color(color, Vec4::new(color.x, color.y, color.z, 0.0))
        .with_scale(6.0, 0.5)
        .with_drag(2.5)
        .with_emissive(2.0)
        .with_texture(tex)
}

/// Larger, omnidirectional explosion for a scored goal.
pub(crate) fn goal_explosion(color: Vec4, theme: &ChaosTheme, tex: u32) -> ParticleConfig {
    let count = (80.0 * theme.particle_count_mult).round() as usize;
    ParticleConfig::burst(count)
        .with_lifetime(0.4, 1.0)
        .with_speed(120.0, 520.0)
        .with_direction(Vec2::Y, std::f32::consts::PI) // full circle
        .with_color(color, Vec4::new(color.x, color.y, color.z, 0.0))
        .with_scale(10.0, 0.5)
        .with_drag(1.6)
        .with_emissive(3.0)
        .with_texture(tex)
}
