# Insiculous Pong

Neon Pong built on the [insiculous_2d](../../insiculous_2d) engine — bloom-heavy
Geometry-Wars look, a spring-mass deforming grid background, power-ups,
achievements, and the engine's signature chaos modes.

## Running

The game depends on the engine by path (`../../insiculous_2d`), so keep both
checkouts side by side:

```bash
cargo run                     # play the game
cargo run --features editor   # run the game inside the engine's visual editor
```

Assets and saves resolve relative to the executable (falling back to the crate
directory), so `cargo run` works from any working directory. Achievements
persist to `saves/pong_achievements.json`.

## Controls

**Menus** — `W`/`S` or `↑`/`↓` to navigate, `Enter`/`Space` to confirm,
`Escape` to go back.

| Mode | Left paddle | Right paddle |
|------|-------------|--------------|
| Single player | `W`/`S` or `↑`/`↓` | AI (Easy / Medium / Hard) |
| Two player | `W`/`S` | `↑`/`↓` |

`F1` during a match toggles the in-game collider debug overlay (magenta
outlines drawn over the sprites).

## Chaos Modes

Pick one before each match:

| Mode | Effect |
|------|--------|
| Normal | Classic Pong |
| Insane | Ball speeds up on every paddle hit |
| Ridiculous | Match starts with two balls |
| Insiculous | Both at once |

## Editor Mode

`cargo run --features editor` opens the exact same game inside the engine's
scene editor — useful for inspecting and tuning entities while the game runs:

- **Play / Pause / Stop**: `F5` or `Ctrl+P` to play, `Ctrl+P` to pause,
  `Ctrl+Shift+P` to stop and restore the pre-play world.
- **Inspect**: click entities in the hierarchy or viewport; the inspector
  shows and edits Transform2D, Sprite, RigidBody, and Collider fields with
  undo/redo (`Ctrl+Z`/`Ctrl+Y`).
- **Collider overlay**: press `C` to toggle collider outlines in the scene
  view (green = solid, cyan = sensors like the goal zones, yellow = selected).
  The outlines show exactly what the physics simulation uses — collider sizes
  are absolute pixels and ignore `Transform2D.scale`, which is how the sprites
  are sized, so any sprite-vs-collider mismatch is immediately visible.
- **Tune collider shapes**: box half-extents, circle radius, and capsule
  height/radius are editable in the inspector. The overlay updates live;
  the running simulation picks the new shape up when the body is next
  created (e.g. a fresh play session). For permanent fixes, copy the tuned
  values back into `src/constants.rs` (`PADDLE_W`, `PADDLE_H`, `BALL_SIZE`,
  ...), since all entities are spawned from those constants in
  `src/spawning.rs`.

## Project Layout

```
src/
├── main.rs          # Game trait impl, window/config setup, editor wiring
├── constants.rs     # All gameplay tuning values (sizes, speeds, layout)
├── types.rs         # PongGame state (Playfield, Balls, Scoreboard, ...) and enums
├── spawning.rs      # All entity creation, each entity Named for the editor
├── gameplay/
│   ├── mod.rs       # Match update loop orchestration, grid step/ripple
│   ├── paddles.rs   # Player paddle control and CPU AI
│   ├── balls.rs     # Ball speed maintenance, extra-ball spawn/teardown
│   ├── scoring.rs   # Goal detection, point awards, win condition
│   └── flow.rs      # Serve/game-over input, match start/reset, visibility
├── menu.rs          # Title / difficulty / chaos / achievements navigation
├── power_ups.rs     # Power-up timing and pickup effects
├── effects.rs       # Deforming grid background, hit effects
├── chaos_theme.rs   # Per-chaos-mode color themes
├── achievements.rs  # Achievement definitions
└── ui.rs            # Menu screens and in-match HUD text
```

Every spawned entity carries a `Name` component ("Left Paddle", "Ball",
"Top Wall", "Power-Up (Multi-Ball)", ...), so the editor hierarchy shows
readable names instead of `Entity 7`.
