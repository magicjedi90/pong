use engine_core::prelude::*;

pub(crate) const WIN_W: f32 = 800.0;
pub(crate) const WIN_H: f32 = 600.0;

// The renderer multiplies Transform2D.scale by 80 to get pixel size.
const RENDER_UNIT: f32 = 80.0;

pub(crate) const PADDLE_W: f32 = 20.0;
pub(crate) const PADDLE_H: f32 = 120.0;
pub(crate) const PADDLE_SCALE: Vec2 = Vec2::new(PADDLE_W / RENDER_UNIT, PADDLE_H / RENDER_UNIT);
pub(crate) const PADDLE_X: f32 = 370.0;
pub(crate) const PADDLE_MAX_Y: f32 = WIN_H / 2.0 - PADDLE_H / 2.0 - 10.0;
pub(crate) const PADDLE_SPEED: f32 = 450.0;

const BALL_SIZE: f32 = 20.0;
pub(crate) const BALL_SCALE: f32 = BALL_SIZE / RENDER_UNIT;
pub(crate) const BALL_RADIUS: f32 = BALL_SIZE / 2.0;
pub(crate) const BALL_INITIAL_SPEED: f32 = 250.0;
pub(crate) const BALL_MAX_SPEED: f32 = 500.0;

pub(crate) const WIN_SCORE: u32 = 7;

pub(crate) const LEFT_COLOR: Vec4 = Vec4::new(1.0, 0.3, 0.3, 1.0);
pub(crate) const RIGHT_COLOR: Vec4 = Vec4::new(0.3, 0.5, 1.0, 1.0);
// Power-ups
pub(crate) const POWERUP_SIZE: f32 = 24.0;
pub(crate) const POWERUP_SCALE: f32 = POWERUP_SIZE / RENDER_UNIT;
pub(crate) const SPEED_BOOST_COLOR: Vec4 = Vec4::new(0.2, 1.0, 0.3, 1.0);
pub(crate) const MULTIBALL_COLOR: Vec4 = Vec4::new(1.0, 1.0, 0.2, 1.0);
pub(crate) const SPEED_BOOST_DURATION: f32 = 5.0;
pub(crate) const SPEED_BOOST_MULTIPLIER: f32 = 1.8;
pub(crate) const POWERUP_SPAWN_MIN: f32 = 5.0;
pub(crate) const POWERUP_SPAWN_MAX: f32 = 12.0;
pub(crate) const POWERUP_INITIAL_DELAY: f32 = 8.0;
pub(crate) const MAX_POWERUPS: usize = 3;
