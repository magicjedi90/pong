use engine_core::prelude::*;
use crate::constants::*;
use crate::types::PongGame;

pub(crate) fn spawn_paddle(world: &mut World, x: f32, tex: u32) -> EntityId {
    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::from_parts(Vec2::new(x, 0.0), 0.0, PADDLE_SCALE)).ok();
    world.add_component(&entity, Sprite::new(tex)).ok();
    world.add_component(&entity, RigidBody::new_kinematic().with_rotation_locked(true)).ok();
    world.add_component(&entity, Collider::box_collider(PADDLE_W, PADDLE_H).with_friction(0.0).with_restitution(1.0)).ok();
    entity
}

pub(crate) fn spawn_wall(world: &mut World, pos: Vec2, w: f32, h: f32) {
    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(pos)).ok();
    world.add_component(&entity, RigidBody::new_static()).ok();
    world.add_component(&entity, Collider::box_collider(w, h).with_friction(0.0).with_restitution(1.0)).ok();
}

pub(crate) fn spawn_goal_sensor(world: &mut World, x: f32) -> EntityId {
    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::new(x, 0.0))).ok();
    world.add_component(&entity, RigidBody::new_static()).ok();
    world.add_component(&entity, Collider::box_collider(20.0, WIN_H).as_sensor()).ok();
    entity
}

impl PongGame {
    pub(crate) fn spawn_ball(&self, world: &mut World, tex: u32) -> EntityId {
        let ball = world.create_entity();
        world.add_component(&ball, Transform2D::from_parts(Vec2::ZERO, 0.0, Vec2::splat(BALL_SCALE))).ok();
        world.add_component(&ball, Sprite::new(tex)).ok();
        world.add_component(&ball,
            RigidBody::new_dynamic()
                .with_gravity_scale(0.0)
                .with_rotation_locked(true)
                .with_linear_damping(0.0)
                .with_angular_damping(0.0)
                .with_ccd(true),
        ).ok();
        world.add_component(&ball,
            Collider::circle_collider(BALL_RADIUS)
                .with_friction(0.0)
                .with_restitution(1.0),
        ).ok();
        ball
    }
}
