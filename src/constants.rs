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
