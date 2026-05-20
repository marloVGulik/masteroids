# Masteroids

Multiplayer Asteroids game built in Rust with egui.

## Build & Run

```bash
cargo run
```

## Controls

| Key       | Action        |
|-----------|---------------|
| W         | Thrust forward|
| A         | Turn left     |
| D         | Turn right    |
| Space     | Shoot         |

## Screens

- **Start** — enter hostname and username, choose to play or host
- **Host** — listens for connections, manages the game state
- **Player** — joins a game or plays single-player
- **Settings** — placeholder for future configuration

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for a detailed overview.
