use engine_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

const WIN_W: f32 = 800.0;
const WIN_H: f32 = 600.0;

// The renderer multiplies Transform2D.scale by 80 to get pixel size.
const RENDER_UNIT: f32 = 80.0;

const PADDLE_W: f32 = 20.0;
const PADDLE_H: f32 = 120.0;
const PADDLE_SCALE: Vec2 = Vec2::new(PADDLE_W / RENDER_UNIT, PADDLE_H / RENDER_UNIT);
const PADDLE_X: f32 = 370.0;
const PADDLE_MAX_Y: f32 = WIN_H / 2.0 - PADDLE_H / 2.0 - 10.0;
const PADDLE_SPEED: f32 = 450.0;
const AI_SPEED: f32 = 300.0 * 0.85;
const AI_DEAD_ZONE: f32 = 2.0;

const BALL_SIZE: f32 = 20.0;
const BALL_SCALE: f32 = BALL_SIZE / RENDER_UNIT;
const BALL_RADIUS: f32 = BALL_SIZE / 2.0;
const BALL_INITIAL_SPEED: f32 = 250.0;
const BALL_MAX_SPEED: f32 = 500.0;

const WIN_SCORE: u32 = 7;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum Side { Left, Right }

#[derive(Debug, Clone, PartialEq)]
enum GameState { Serving, Playing, GameOver { player_wins: bool } }

struct PongGame {
    physics: PhysicsSystem,

    left_paddle: Option<EntityId>,
    right_paddle: Option<EntityId>,
    ball: Option<EntityId>,
    left_goal: Option<EntityId>,
    right_goal: Option<EntityId>,

    score_left: u32,
    score_right: u32,
    last_scorer: Side,
    state: GameState,
    frame_count: u32,
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
            last_scorer: Side::Right, // first serve goes toward player
            state: GameState::Serving,
            frame_count: 0,
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn entity_y(world: &World, entity: EntityId) -> f32 {
    world.get::<Transform2D>(entity).map(|t| t.position.y).unwrap_or(0.0)
}

fn collision_involves(event: &CollisionEvent, a: EntityId, b: EntityId) -> bool {
    (event.entity_a == a && event.entity_b == b)
        || (event.entity_a == b && event.entity_b == a)
}

fn spawn_paddle(world: &mut World, x: f32, tex: u32) -> EntityId {
    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::from_parts(Vec2::new(x, 0.0), 0.0, PADDLE_SCALE)).ok();
    world.add_component(&entity, Sprite::new(tex)).ok();
    world.add_component(&entity, RigidBody::new_kinematic().with_rotation_locked(true)).ok();
    world.add_component(&entity, Collider::box_collider(PADDLE_W, PADDLE_H).with_friction(0.0).with_restitution(1.0)).ok();
    entity
}

fn spawn_wall(world: &mut World, pos: Vec2, w: f32, h: f32) {
    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(pos)).ok();
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

// ─── Game trait ──────────────────────────────────────────────────────────────

impl Game for PongGame {
    fn init(&mut self, ctx: &mut GameContext) {
        if let Ok(font) = ctx.ui.load_font_file("assets/fonts/font.ttf") {
            ctx.ui.set_default_font(font);
        }

        let tex = ctx.assets.create_solid_color(1, 1, [255, 255, 255, 255]).unwrap();

        self.left_paddle = Some(spawn_paddle(&mut ctx.world, -PADDLE_X, tex.id));
        self.right_paddle = Some(spawn_paddle(&mut ctx.world, PADDLE_X, tex.id));
        self.ball = Some(self.spawn_ball(&mut ctx.world, tex.id));

        let wall_y = WIN_H / 2.0 - 10.0;
        spawn_wall(&mut ctx.world, Vec2::new(0.0, wall_y), WIN_W, 20.0);
        spawn_wall(&mut ctx.world, Vec2::new(0.0, -wall_y), WIN_W, 20.0);

        let goal_x = WIN_W / 2.0 + 10.0;
        self.left_goal = Some(spawn_goal_sensor(&mut ctx.world, -goal_x));
        self.right_goal = Some(spawn_goal_sensor(&mut ctx.world, goal_x));
    }

    fn update(&mut self, ctx: &mut GameContext) {
        let ball = match self.ball { Some(e) => e, None => return };
        let left_paddle = match self.left_paddle { Some(e) => e, None => return };
        let right_paddle = match self.right_paddle { Some(e) => e, None => return };

        self.update_player_paddle(ctx, left_paddle);
        self.update_ai_paddle(ctx, right_paddle, ball);
        self.physics.update(&mut ctx.world, ctx.delta_time);

        self.frame_count = self.frame_count.wrapping_add(1);
        self.handle_input(ctx, ball);
        self.maintain_ball_velocity(ball);
        self.check_goals(ball);
        self.check_win_condition();
        self.draw_ui(ctx);
    }
}

// ─── Game logic ──────────────────────────────────────────────────────────────

impl PongGame {
    fn spawn_ball(&self, world: &mut World, tex: u32) -> EntityId {
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

    fn update_player_paddle(&mut self, ctx: &GameContext, paddle: EntityId) {
        let up = ctx.input.is_key_pressed(KeyCode::KeyW) || ctx.input.is_key_pressed(KeyCode::ArrowUp);
        let down = ctx.input.is_key_pressed(KeyCode::KeyS) || ctx.input.is_key_pressed(KeyCode::ArrowDown);
        let dy = match (up, down) {
            (true, false) => PADDLE_SPEED,
            (false, true) => -PADDLE_SPEED,
            _ => 0.0,
        };
        let y = entity_y(&ctx.world, paddle);
        let new_y = (y + dy * ctx.delta_time).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);
        self.physics.physics_world_mut().set_kinematic_target(paddle, Vec2::new(-PADDLE_X, new_y), 0.0);
    }

    fn update_ai_paddle(&mut self, ctx: &GameContext, paddle: EntityId, ball: EntityId) {
        let ball_y = entity_y(&ctx.world, ball);
        let paddle_y = entity_y(&ctx.world, paddle);
        let diff = ball_y - paddle_y;
        let dy = if diff.abs() > AI_DEAD_ZONE { diff.signum() * AI_SPEED } else { 0.0 };
        let new_y = (paddle_y + dy * ctx.delta_time).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);
        self.physics.physics_world_mut().set_kinematic_target(paddle, Vec2::new(PADDLE_X, new_y), 0.0);
    }

    fn handle_input(&mut self, ctx: &mut GameContext, ball: EntityId) {
        if !ctx.input.is_key_just_pressed(KeyCode::Space) { return; }

        match &self.state {
            GameState::Serving => {
                let dir_x = match self.last_scorer {
                    Side::Left => -1.0,
                    Side::Right => 1.0,
                };
                let hash = self.frame_count.wrapping_mul(2654435761);
                let t = ((hash >> 16) as f32) / 65535.0;
                let dir_y = t * 1.2 - 0.6;
                let dir = Vec2::new(dir_x, dir_y).normalize();
                self.physics.apply_impulse(ball, dir * BALL_INITIAL_SPEED);
                self.state = GameState::Playing;
            }
            GameState::GameOver { .. } => {
                self.score_left = 0;
                self.score_right = 0;
                self.last_scorer = Side::Right;
                self.reset_ball();
            }
            GameState::Playing => {}
        }
    }

    fn maintain_ball_velocity(&mut self, ball: EntityId) {
        let Some((vel, _)) = self.physics.physics_world().get_body_velocity(ball) else { return };
        if vel.x.abs() < 0.1 { return; }

        let fixed_vx = vel.x.signum() * BALL_INITIAL_SPEED;
        let vy = vel.y.clamp(-BALL_MAX_SPEED, BALL_MAX_SPEED);
        let new_vel = Vec2::new(fixed_vx, vy);

        if (new_vel - vel).length() > 1.0 {
            self.physics.physics_world_mut().set_body_velocity(ball, new_vel, 0.0);
        }
    }

    fn check_goals(&mut self, ball: EntityId) {
        let left_goal = match self.left_goal { Some(e) => e, None => return };
        let right_goal = match self.right_goal { Some(e) => e, None => return };

        let mut scored: Option<Side> = None;
        for collision in self.physics.collision_events() {
            if !collision.event.started { continue; }
            if collision_involves(&collision.event, ball, left_goal) {
                scored = Some(Side::Right);
            } else if collision_involves(&collision.event, ball, right_goal) {
                scored = Some(Side::Left);
            }
        }

        if let Some(side) = scored {
            match side {
                Side::Left => self.score_left += 1,
                Side::Right => self.score_right += 1,
            }
            self.last_scorer = side;
            self.reset_ball();
        }
    }

    fn check_win_condition(&mut self) {
        if !matches!(self.state, GameState::Playing | GameState::Serving) { return; }

        if self.score_left >= WIN_SCORE {
            self.state = GameState::GameOver { player_wins: true };
        } else if self.score_right >= WIN_SCORE {
            self.state = GameState::GameOver { player_wins: false };
        }
    }

    fn draw_ui(&self, ctx: &mut GameContext) {
        ctx.ui.begin_frame(ctx.input, ctx.window_size);

        let cx = ctx.window_size.x / 2.0;
        let cy = ctx.window_size.y / 2.0;

        ctx.ui.label(&format!("{}  :  {}", self.score_left, self.score_right), Vec2::new(cx - 40.0, 24.0));

        match &self.state {
            GameState::Serving => {
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

    fn reset_ball(&mut self) {
        if let Some(ball) = self.ball {
            self.physics.physics_world_mut().set_body_transform(ball, Vec2::ZERO, 0.0);
            self.physics.physics_world_mut().set_body_velocity(ball, Vec2::ZERO, 0.0);
        }
        self.state = GameState::Serving;
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
