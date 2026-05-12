use engine_core::prelude::*;
use crate::constants::*;
use crate::types::PongGame;

pub(crate) fn spawn_paddle(world: &mut World, x: f32, tex: u32, color: Vec4) -> EntityId {
    world.spawn()
        .with(Transform2D::from_parts(Vec2::new(x, 0.0), 0.0, PADDLE_SCALE))
        // Paddles glow strongly so bloom traces a halo around them — the
        // signature Geometry-Wars treatment for player-controlled objects.
        .with(Sprite::new(tex).with_color(color).with_emissive(1.5))
        .with(RigidBody::new_kinematic().with_rotation_locked(true))
        .with(Collider::box_collider(PADDLE_W, PADDLE_H).with_friction(0.0).with_restitution(1.0))
        .id()
}

pub(crate) fn spawn_wall(world: &mut World, pos: Vec2, w: f32, h: f32, tex: u32, color: Vec4) -> EntityId {
    const RENDER_UNIT: f32 = 80.0;
    world.spawn()
        .with(Transform2D::from_parts(pos, 0.0, Vec2::new(w / RENDER_UNIT, h / RENDER_UNIT)))
        // Walls glow gently — they outline the playfield without dominating.
        .with(Sprite::new(tex).with_color(color).with_depth(-1.0).with_emissive(0.6))
        .with(RigidBody::new_static())
        .with(Collider::box_collider(w, h).with_friction(0.0).with_restitution(1.0))
        .id()
}

pub(crate) fn spawn_background(world: &mut World, tex: u32, color: Vec4) -> EntityId {
    const RENDER_UNIT: f32 = 80.0;
    // Slightly oversize so resizes don't reveal a seam.
    let w = crate::constants::WIN_W * 1.2;
    let h = crate::constants::WIN_H * 1.2;
    world.spawn()
        .with(Transform2D::from_parts(Vec2::ZERO, 0.0, Vec2::new(w / RENDER_UNIT, h / RENDER_UNIT)))
        // Background is intentionally non-emissive so the grid lines (drawn
        // on top by the line pipeline) and gameplay sprites pop against it.
        .with(Sprite::new(tex).with_color(color).with_depth(-100.0))
        .id()
}

pub(crate) fn spawn_goal_sensor(world: &mut World, x: f32) -> EntityId {
    world.spawn()
        .with(Transform2D::new(Vec2::new(x, 0.0)))
        .with(RigidBody::new_static())
        .with(Collider::box_collider(20.0, WIN_H).as_sensor())
        .id()
}

impl PongGame {
    pub(crate) fn spawn_ball(&self, world: &mut World, tex: u32) -> EntityId {
        world.spawn()
            .with(Transform2D::from_parts(Vec2::ZERO, 0.0, Vec2::splat(BALL_SCALE)))
            // Ball is the brightest object on screen — high emissive value
            // gives it a strong neon core that smears with motion via bloom.
            .with(Sprite::new(tex).with_emissive(2.5))
            .with(RigidBody::new_dynamic()
                .with_gravity_scale(0.0)
                .with_rotation_locked(true)
                .with_linear_damping(0.0)
                .with_angular_damping(0.0)
                .with_ccd(true))
            .with(Collider::circle_collider(BALL_RADIUS)
                .with_friction(0.0)
                .with_restitution(1.0))
            .id()
    }
}
