//! All entity creation. Every entity gets a `Name` component so the editor
//! hierarchy shows "Left Paddle" instead of "Entity 7".

use engine_core::prelude::*;
use crate::constants::*;
use crate::types::{PongGame, PowerUpKind};

/// Spawn one paddle. The source `paddle_16px.png` has its flat face on the
/// left and rounded face on the right; pass `mirror = true` for the right
/// paddle so its rounded face points back at the opponent.
///
/// Physics uses a capsule collider — the flat center returns balls
/// predictably, while the rounded top/bottom caps produce dramatic angles
/// on edge hits.
pub(crate) fn spawn_paddle(
    world: &mut World,
    name: &str,
    x: f32,
    paddle_tex: u32,
    color: Vec4,
    mirror: bool,
) -> EntityId {
    // The Sprite's scale is multiplied by Transform2D scale at render time;
    // flipping its X sign mirrors the rendered texture without affecting the
    // collider (which uses absolute PADDLE_W/H).
    let sprite_scale = Vec2::new(if mirror { -1.0 } else { 1.0 }, 1.0);
    world.spawn()
        .with(Name::new(name))
        .with(Transform2D::from_parts(Vec2::new(x, 0.0), 0.0, PADDLE_SCALE))
        // Paddles glow strongly so bloom traces a halo around them — the
        // signature Geometry-Wars treatment for player-controlled objects.
        .with(Sprite::new(paddle_tex)
            .with_color(color)
            .with_emissive(1.5)
            .with_scale(sprite_scale))
        .with(RigidBody::new_kinematic().with_rotation_locked(true))
        // Capsule: total height = PADDLE_H, cap radius = PADDLE_W/2. Flat
        // body is the cylindrical middle, rounded caps live at top and bottom.
        .with(Collider::new(ColliderShape::capsule_y(PADDLE_H, PADDLE_W * 0.5))
            .with_friction(0.0)
            .with_restitution(1.0))
        .id()
}

pub(crate) fn spawn_wall(
    world: &mut World,
    name: &str,
    pos: Vec2,
    w: f32,
    h: f32,
    tex: u32,
    color: Vec4,
) -> EntityId {
    const RENDER_UNIT: f32 = 80.0;
    world.spawn()
        .with(Name::new(name))
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
        .with(Name::new("Background"))
        .with(Transform2D::from_parts(Vec2::ZERO, 0.0, Vec2::new(w / RENDER_UNIT, h / RENDER_UNIT)))
        // Background is intentionally non-emissive so the grid lines (drawn
        // on top by the line pipeline) and gameplay sprites pop against it.
        .with(Sprite::new(tex).with_color(color).with_depth(-100.0))
        .id()
}

pub(crate) fn spawn_goal_sensor(world: &mut World, name: &str, x: f32) -> EntityId {
    world.spawn()
        .with(Name::new(name))
        .with(Transform2D::new(Vec2::new(x, 0.0)))
        .with(RigidBody::new_static())
        .with(Collider::box_collider(20.0, WIN_H).as_sensor())
        .id()
}

impl PongGame {
    /// Spawn a ball entity using the loaded ball PNG texture. The collider
    /// stays a true circle so reflection physics off the paddle's capsule
    /// caps matches what the player sees on screen.
    pub(crate) fn spawn_ball(&self, world: &mut World, name: &str) -> EntityId {
        world.spawn()
            .with(Name::new(name))
            .with(Transform2D::from_parts(Vec2::ZERO, 0.0, Vec2::splat(BALL_SCALE)))
            // Ball is the brightest object on screen — high emissive value
            // gives it a strong neon core that smears with motion via bloom.
            .with(Sprite::new(self.textures.ball).with_emissive(2.5))
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

    /// Editor-hierarchy name for the next extra ball ("Ball 2", "Ball 3", ...).
    pub(crate) fn next_extra_ball_name(&self) -> String {
        format!("Ball {}", self.balls.extras.len() + 2)
    }

    /// Spawn a power-up pickup at `pos` and track it in `self.power_ups`.
    pub(crate) fn spawn_power_up(&mut self, world: &mut World, kind: PowerUpKind, pos: Vec2) {
        let color = match kind {
            PowerUpKind::SpeedBoost => SPEED_BOOST_COLOR,
            PowerUpKind::MultiBall => MULTIBALL_COLOR,
        };
        let entity = world.spawn()
            .with(Name::new(kind.entity_name()))
            .with(Transform2D::from_parts(pos, 0.0, Vec2::splat(POWERUP_SCALE)))
            .with(Sprite::new(self.textures.white).with_color(color))
            .with(RigidBody::new_static())
            .with(Collider::circle_collider(POWERUP_SIZE / 2.0).as_sensor())
            .id();
        self.power_ups.active.track(entity, kind);
    }
}
