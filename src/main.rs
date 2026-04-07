use engine_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

const WIN_W: f32 = 800.0;
const WIN_H: f32 = 600.0;

// The renderer multiplies Transform2D.scale by 80 to get pixel size.
// So scale = pixels / 80.
const UNIT: f32 = 80.0;

const PADDLE_W: f32 = 20.0;  // pixel width
const PADDLE_H: f32 = 120.0; // pixel height
const PADDLE_SCALE_X: f32 = PADDLE_W / UNIT;
const PADDLE_SCALE_Y: f32 = PADDLE_H / UNIT;
const PADDLE_X: f32 = 370.0;
const PADDLE_MAX_Y: f32 = WIN_H / 2.0 - PADDLE_H / 2.0 - 10.0; // stay inside walls
const PADDLE_SPEED: f32 = 300.0;

const BALL_SIZE: f32 = 20.0;  // pixel diameter
const BALL_SCALE: f32 = BALL_SIZE / UNIT;
const BALL_RADIUS: f32 = BALL_SIZE / 2.0; // physics collider radius in world units
const BALL_INITIAL_SPEED: f32 = 250.0;
const BALL_MAX_SPEED: f32 = 500.0;

const WIN_SCORE: u32 = 7;

// ─── Game state ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum GameState {
    Serving { left_serves: bool },
    Playing,
    GameOver { player_wins: bool },
}

struct PongGame {
    physics: PhysicsSystem,

    left_paddle: Option<EntityId>,
    right_paddle: Option<EntityId>,
    ball: Option<EntityId>,
    left_goal: Option<EntityId>,
    right_goal: Option<EntityId>,

    score_left: u32,
    score_right: u32,
    state: GameState,

    white_tex: u32,
}

impl Default for PongGame {
    fn default() -> Self {
        Self {
            physics: PhysicsSystem::with_config(PhysicsConfig::top_down()),
            left_paddle: None,
            right_paddle: None,
            ball: None,
            left_goal: None,
            right_goal: None,
            score_left: 0,
            score_right: 0,
            state: GameState::Serving { left_serves: true },
            white_tex: 0,
        }
    }
}

// ─── Spawn helpers ────────────────────────────────────────────────────────────

fn spawn_paddle(world: &mut World, x: f32, tex: u32) -> EntityId {
    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::from_parts(Vec2::new(x, 0.0), 0.0, Vec2::new(PADDLE_SCALE_X, PADDLE_SCALE_Y))).ok();
    world.add_component(&entity, Sprite::new(tex)).ok();
    world.add_component(&entity, RigidBody::new_kinematic().with_rotation_locked(true)).ok();
    world.add_component(&entity, Collider::box_collider(PADDLE_W, PADDLE_H).with_friction(0.0).with_restitution(0.0)).ok();
    entity
}

fn spawn_wall(world: &mut World, x: f32, y: f32, w: f32, h: f32) {
    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::new(x, y))).ok();
    world.add_component(&entity, RigidBody::new_static()).ok();
    world.add_component(&entity, Collider::box_collider(w, h).with_friction(0.0).with_restitution(1.0)).ok();
}

fn spawn_goal_sensor(world: &mut World, x: f32) -> EntityId {
    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::new(x, 0.0))).ok();
    world.add_component(&entity, RigidBody::new_static()).ok();
    world.add_component(&entity, Collider::box_collider(20.0, WIN_H).as_sensor()).ok();
    entity
}

// ─── Game impl ────────────────────────────────────────────────────────────────

impl Game for PongGame {
    fn init(&mut self, ctx: &mut GameContext) {
        // Load font for UI text
        if let Ok(font) = ctx.ui.load_font_file("assets/fonts/font.ttf") {
            ctx.ui.set_default_font(font);
        }

        let tex = ctx.assets.create_solid_color(1, 1, [255, 255, 255, 255]).unwrap();
        self.white_tex = tex.id;

        // Paddles
        self.left_paddle = Some(spawn_paddle(&mut ctx.world, -PADDLE_X, tex.id));
        self.right_paddle = Some(spawn_paddle(&mut ctx.world, PADDLE_X, tex.id));

        // Ball (dynamic, zero gravity, CCD so it doesn't tunnel through paddles)
        let ball = ctx.world.create_entity();
        ctx.world.add_component(&ball, Transform2D::from_parts(Vec2::ZERO, 0.0, Vec2::splat(BALL_SCALE))).ok();
        ctx.world.add_component(&ball, Sprite::new(tex.id)).ok();
        ctx.world.add_component(&ball,
            RigidBody::new_dynamic()
                .with_gravity_scale(0.0)
                .with_rotation_locked(true)
                .with_linear_damping(0.0)
                .with_angular_damping(0.0)
                .with_ccd(true),
        ).ok();
        ctx.world.add_component(&ball,
            Collider::circle_collider(BALL_RADIUS)
                .with_friction(0.0)
                .with_restitution(1.0),
        ).ok();
        self.ball = Some(ball);

        // Walls (top and bottom)
        spawn_wall(&mut ctx.world, 0.0, WIN_H / 2.0 - 10.0, WIN_W, 20.0);
        spawn_wall(&mut ctx.world, 0.0, -(WIN_H / 2.0 - 10.0), WIN_W, 20.0);

        // Goal sensors just outside the screen edges
        self.left_goal = Some(spawn_goal_sensor(&mut ctx.world, -(WIN_W / 2.0 + 10.0)));
        self.right_goal = Some(spawn_goal_sensor(&mut ctx.world, WIN_W / 2.0 + 10.0));
    }

    fn update(&mut self, ctx: &mut GameContext) {
        let ball = match self.ball { Some(e) => e, None => return };
        let left_paddle = match self.left_paddle { Some(e) => e, None => return };
        let right_paddle = match self.right_paddle { Some(e) => e, None => return };
        let left_goal = match self.left_goal { Some(e) => e, None => return };
        let right_goal = match self.right_goal { Some(e) => e, None => return };

        // ── Left paddle (player: W/S) ──────────────────────────────────────────
        let mut left_dy = 0.0f32;
        if ctx.input.is_key_pressed(KeyCode::KeyW) { left_dy = PADDLE_SPEED; }
        if ctx.input.is_key_pressed(KeyCode::KeyS) { left_dy = -PADDLE_SPEED; }

        let left_y = ctx.world.get::<Transform2D>(left_paddle).map(|t| t.position.y).unwrap_or(0.0);
        let new_left_y = (left_y + left_dy * ctx.delta_time).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);
        self.physics.physics_world_mut().set_kinematic_target(left_paddle, Vec2::new(-PADDLE_X, new_left_y), 0.0);

        // ── Right paddle (AI: tracks ball Y) ──────────────────────────────────
        let ball_y = ctx.world.get::<Transform2D>(ball).map(|t| t.position.y).unwrap_or(0.0);
        let right_y = ctx.world.get::<Transform2D>(right_paddle).map(|t| t.position.y).unwrap_or(0.0);
        let ai_dy = (ball_y - right_y).clamp(-PADDLE_SPEED, PADDLE_SPEED);
        let new_right_y = (right_y + ai_dy * ctx.delta_time).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);
        self.physics.physics_world_mut().set_kinematic_target(right_paddle, Vec2::new(PADDLE_X, new_right_y), 0.0);

        // ── Physics step (must run before serve so bodies are synced to rapier) ─
        self.physics.update(&mut ctx.world, ctx.delta_time);

        // ── Serving / game over input (after physics so bodies exist in rapier) ─
        if ctx.input.is_key_just_pressed(KeyCode::Space) {
            match &self.state {
                GameState::Serving { left_serves } => {
                    let dir_x = if *left_serves { 1.0 } else { -1.0 };
                    let dir = Vec2::new(dir_x, 0.5).normalize();
                    self.physics.apply_impulse(ball, dir * BALL_INITIAL_SPEED);
                    self.state = GameState::Playing;
                }
                GameState::GameOver { .. } => {
                    self.score_left = 0;
                    self.score_right = 0;
                    self.reset_ball(true);
                }
                GameState::Playing => {}
            }
        }

        // ── Speed cap (prevents ball from accelerating indefinitely) ──────────
        if let Some((vel, _)) = self.physics.physics_world().get_body_velocity(ball) {
            let speed = vel.length();
            if speed > BALL_MAX_SPEED && speed > 0.0 {
                self.physics.physics_world_mut().set_body_velocity(ball, vel.normalize() * BALL_MAX_SPEED, 0.0);
            }
        }

        // ── Collision events: goal detection ──────────────────────────────────
        let mut left_scored = false;
        let mut right_scored = false;

        for collision in self.physics.collision_events() {
            if !collision.event.started { continue; }
            let a = collision.event.entity_a;
            let b = collision.event.entity_b;

            // Ball enters left goal → right player scores
            if (a == ball && b == left_goal) || (b == ball && a == left_goal) {
                right_scored = true;
            }
            // Ball enters right goal → left player scores
            if (a == ball && b == right_goal) || (b == ball && a == right_goal) {
                left_scored = true;
            }
        }

        if right_scored {
            self.score_right += 1;
            self.reset_ball(false); // right side serves next
        }
        if left_scored {
            self.score_left += 1;
            self.reset_ball(true); // left side serves next
        }

        // ── Win condition ──────────────────────────────────────────────────────
        if matches!(self.state, GameState::Playing | GameState::Serving { .. }) {
            if self.score_left >= WIN_SCORE {
                self.state = GameState::GameOver { player_wins: true };
            } else if self.score_right >= WIN_SCORE {
                self.state = GameState::GameOver { player_wins: false };
            }
        }

        // ── UI ────────────────────────────────────────────────────────────────
        ctx.ui.begin_frame(ctx.input, ctx.window_size);

        let cx = ctx.window_size.x / 2.0;
        let cy = ctx.window_size.y / 2.0;

        // Scores near top center
        ctx.ui.label(&format!("{}", self.score_left), Vec2::new(cx - 50.0, 24.0));
        ctx.ui.label(":", Vec2::new(cx - 8.0, 24.0));
        ctx.ui.label(&format!("{}", self.score_right), Vec2::new(cx + 20.0, 24.0));

        // State messages
        match &self.state {
            GameState::Serving { .. } => {
                ctx.ui.label("Press SPACE to serve", Vec2::new(cx - 88.0, cy - 10.0));
            }
            GameState::GameOver { player_wins } => {
                let msg = if *player_wins { "PLAYER WINS!" } else { "CPU WINS!" };
                ctx.ui.label(msg, Vec2::new(cx - 56.0, cy - 20.0));
                ctx.ui.label("Press SPACE to restart", Vec2::new(cx - 96.0, cy + 10.0));
            }
            GameState::Playing => {}
        }

        ctx.ui.end_frame();
    }
}

impl PongGame {
    fn reset_ball(&mut self, left_serves: bool) {
        if let Some(ball) = self.ball {
            self.physics.physics_world_mut().set_body_transform(ball, Vec2::ZERO, 0.0);
            self.physics.physics_world_mut().set_body_velocity(ball, Vec2::ZERO, 0.0);
        }
        self.state = GameState::Serving { left_serves };
    }
}

// ─── Entry point ──────────────────────────────────────────────────────────────

fn main() {
    let config = GameConfig::new("Pong")
        .with_size(WIN_W as u32, WIN_H as u32)
        .with_clear_color(0.0, 0.0, 0.0, 1.0)
        .with_fps(60);

    run_game(PongGame::default(), config).unwrap();
}
