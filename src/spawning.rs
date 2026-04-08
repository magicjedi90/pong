use engine_core::prelude::*;
use crate::constants::*;
use crate::types::PongGame;

pub(crate) fn spawn_paddle(world: &mut World, x: f32, tex: u32, color: Vec4) -> EntityId {
    world.spawn()
        .with(Transform2D::from_parts(Vec2::new(x, 0.0), 0.0, PADDLE_SCALE))
        .with(Sprite::new(tex).with_color(color))
        .with(RigidBody::new_kinematic().with_rotation_locked(true))
        .with(Collider::box_collider(PADDLE_W, PADDLE_H).with_friction(0.0).with_restitution(1.0))
        .id()
}

pub(crate) fn spawn_wall(world: &mut World, pos: Vec2, w: f32, h: f32) {
    world.spawn()
        .with(Transform2D::new(pos))
        .with(RigidBody::new_static())
        .with(Collider::box_collider(w, h).with_friction(0.0).with_restitution(1.0));
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
            .with(Sprite::new(tex))
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
