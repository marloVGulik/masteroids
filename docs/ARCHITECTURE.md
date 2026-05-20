//! # Masteroids Architecture

## Overview

Masteroids is a multiplayer Asteroids game built with Rust and egui (via eframe).
The codebase follows a screen-based navigation pattern where each screen is a trait
object managing its own state and rendering.

## Module Structure

```
src/
├── lib.rs          # Public exports (App, Screen, game, core)
├── main.rs         # Entry point: runs the eframe app
├── app.rs          # Top-level eframe application struct
├── screen.rs       # Screen trait + ScreenCommand enum (navigation)
├── states.rs       # Unused legacy state enum (keep for reference)
├── core/           # Shared subsystems
│   ├── mod.rs
│   ├── gameobject.rs  # GameObject + Collidable traits
│   ├── physics.rs     # Collision detection helpers
│   ├── scheduler.rs   # Generic timed task scheduler
│   └── networking.rs  # UDP message protocol + NetworkManager
├── game/           # Single-player game logic
│   ├── mod.rs
│   ├── game.rs      # Game struct, GameInput, GameEvent, GameState
│   └── objects/
│       ├── mod.rs
│       ├── ship.rs  # Player ship (position, velocity, rotation, draw)
│       ├── asteroid.rs  # Asteroid entity + split logic
│       └── bullet.rs    # Bullet entity with lifetime
└── screens/        # UI screens
    ├── mod.rs
    ├── start.rs     # Main menu
    ├── settings.rs  # Placeholder
    ├── player.rs    # In-game client screen
    └── host.rs      # Multiplayer host screen
```

## Screen Flow

```
Start → Play → Player
      → Host → Host
      → Settings → Settings
```

1. **Start screen** collects hostname and username, then dispatches to Play, Host, or Settings.
2. **Player screen** connects to the host (or runs single-player if hostname is empty), processes
   keyboard input, and renders the game view.
3. **Host screen** listens on UDP port 42069, manages connected users, and broadcasts game state.
4. All screens can navigate back to Start via `ScreenCommand::Start`.

## Game Loop

The `Game` struct in `src/game/game.rs` manages:
- **Ship** — position, velocity, rotation, health
- **Asteroids** — spawning, splitting on hit, inter-asteroid collision
- **Bullets** — lifetime, position, collision with asteroids
- **Scheduler** — timed events (e.g. post-damage immunity window)

Each frame:
1. `Player::update()` reads keyboard input and calls `Game::interact()`
2. `Game::update()` advances all entities and checks collisions
3. Collision events (`GameEvent`) are emitted back to `Player::update()` for network sync
4. `Game::draw()` renders all entities into the egui paint layer

## Networking Protocol

Messages use a leading byte as `MessageId` (0–13), followed by big-endian serialized fields.

| ID | Message          | Direction    | Description                          |
|----|------------------|--------------|--------------------------------------|
| 0  | StartGame        | Host→Client  | Signals game start                   |
| 1  | Ready            | Client→Host  | Toggles ready state                  |
| 2  | Alive            | Bidirectional| Liveness heartbeat                   |
| 3  | ShareSeed        | Host→Client  | Random seeds for determinism         |
| 4  | AsteroidHit      | Client→Host  | Reports asteroid destroyed           |
| 5  | Connect          | Client→Host  | Join request with username           |
| 6  | TargetPlayer     | Client→Host  | Selects opponent for combat          |
| 7  | AttackPlayer     | Client→Host  | Executes attack on target            |
| 8  | UserAmount       | Host→Client  | Total connected player count         |
| 9  | SummonAsteroid   | Host→Client  | Spawn asteroid at position           |
| 10 | UserData         | Bidirectional| Synced player score/health/name      |
| 11 | Reject           | Host→Client  | Connection rejected (reason code)    |
| 12 | Accept           | Host→Client  | Connection accepted (assigned ID)    |
| 13 | PlayerDamaged    | Client→Host  | Reports player took damage           |

## Coordinate System

All game positions use a normalized 0–100 coordinate space that scales to the play area
rect each frame. This makes responsive scaling trivial.

## Physics

- Collision uses circle-based hitboxes (`circle_collision`, `point_in_circle`)
- Asteroid split spawns two new asteroids at ±45-degree offsets from the original velocity
- Velocity reflection uses dot product for proper bounce response
- Friction is applied each frame to prevent perpetual acceleration
