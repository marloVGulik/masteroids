# Masteroids Code Review TODOs

## Critical

### ~~#1~~ ~~Fix `Scheduler::update` retain logic silently drops one-shot tasks~~ ✅ Done

**File:** `src/core/scheduler.rs:50`

**Problem:** The retain at the end of `update()` deletes every non-repeating task immediately after it fires. Since `has_ran` is set inside the handler closure and the retain runs in the same call, the task is removed before it can ever fire again. This works accidentally for the current use case (immunity timer fires once and dies), but is a latent bug — any scheduler task that needs to fire exactly once and then be kept for inspection will be lost.

**Fix:** Remove the retain from `update()`, add explicit `remove_fired()` method, and call it after `update()` in `Game::update()`.

---

### ~~#2~~ ~~`Game::activate()` is a no-op — game never starts in single-player mode~~ ✅ Done

**File:** `src/game/game.rs:92-100`

**Problem:** `activate()` contained only commented-out code. There was no reliable way to start a single-player game from the Start screen.

**Fix:** Replaced `activate()` with proper initialization (set state to Active, reset ship/bullets/health). Added `Game::spawn_asteroid()` method. Created new `SinglePlayer` screen (src/screens/single_player.rs) with its own pressure ramping scheduler and runtime toggle. Added `ScreenCommand::SinglePlayer` variant. Updated `Start` screen with a "Single Player" button. Updated `Player::on_activate()` to remove the single-player fallback and detect the sentinel hostname to navigate to SinglePlayer screen.

---

### ~~#3~~ ~~Host liveness check is backwards — `last_alive` increments every frame regardless~~ ✅ Done

**File:** `src/screens/host.rs:98-104`, `src/screens/host.rs:172-177`

**Problem:** The `CheckAlive` task incremented `last_alive` and sent Alive pings to users every second. The liveness logic conflated "ping sent" with "ping received".

**Fix:** Renamed `last_alive` to `last_seen` on `User` struct. Added `last_seen` reset on *any* inbound message from a user (not just `Alive`). Removed the `emit_all` of `Alive` pings from the `CheckAlive` task. Updated user removal to use `last_seen`. Added pressure ramp toggle to Host UI.

---

### ~~#4~~ ~~`NetworkManager::resolver` silently swallows DNS errors~~ ✅ Done

**File:** `src/core/networking.rs:251-265`

**Problem:** `resolver()` returned `None` on DNS failure with zero logging. `emit()` silently dropped messages if resolution failed.

**Fix:** Added `println!` warnings when DNS resolution fails in `resolver()`, when `emit()` fails, and when `new()` falls back to the default address (127.0.0.1:42069).

---

## High

### #5 `GameEvent::PlayerTarget` is never sent over the network

**File:** `src/game/game.rs:200`, `src/screens/player.rs:183-186`

**Problem:** `Game` emits `GameEvent::PlayerTarget { id }` when a collision or interaction occurs, but the `Player` screen only prints it to stdout. The event has no network path — there's no code that converts this into a `NetworkMessage` to tell the target player they're being targeted.

**Fix:** Either remove the `PlayerTarget` event variant (unused), or have the `Player` screen send a `TargetPlayer` message when this event fires.

---

### #6 `Host::emit_all` broadcasts to the sender themselves

**File:** `src/screens/host.rs:67-71`

**Problem:** When the host sends a message to all users, it includes itself in the iteration. This means the host receives its own messages, which can cause duplicate state updates or unexpected re-processing. For example, `UserData` from the host gets sent back to the host.

**Fix:** Filter out the host's own address or ID when emitting. Alternatively, have `emit_all` take an `exclude_id: Option<u32>` parameter.

---

## Medium

### #7 `NetworkMessage::Ready` uses `u8` instead of `bool`

**File:** `src/core/networking.rs:5`, `src/core/networking.rs:112`

**Problem:** `is_ready: u8` accepts any byte value (255, 42, etc.) not just 0 and 1. The deserialization in `from_bytes` treats any non-zero value as true, but the serialization side doesn't enforce the invariant. This could cause inconsistent state if a malformed packet arrives.

**Fix:** Use `bool` and serialize as `u8` internally (0/1), or define a proper `ReadyState { Ready, NotReady }` enum.

---

### #8 `Asteroid::hit_and_copy` mutates its argument in place — confusing API

**File:** `src/game/objects/asteroid.rs:20-43`

**Problem:** `hit_and_copy(&mut self)` mutates the original asteroid (reduces size, changes velocity) and returns a new asteroid. The caller in `game.rs:120` uses it like `Asteroid::hit_and_copy(asteroid)` which is a method call on a mutable reference. This is fine but the name `hit_and_copy` doesn't convey that the original is mutated. It reads like a pure function.

**Fix:** Rename to `split(&mut self) -> Self` to make the mutation clear and match the semantics.

---

### #9 `Player.hostname` is misleading — it's the server address, not a local hostname

**File:** `src/screens/player.rs:17`

**Problem:** The field `hostname` in the `Player` struct is actually the address of the game host server. The name suggests a local machine hostname rather than a remote endpoint. This is confusing when reading the code — `Player::new(hostname, username)` doesn't make it clear that `hostname` is remote.

**Fix:** Rename to `host_address` in both `Player` and the `ScreenCommand::Play { hostname, username }` variant.

---

### #10 `UserData` can't distinguish "no target" from "targeting player 0"

**File:** `src/core/networking.rs:14`, `src/screens/host.rs:217-224`

**Problem:** `User.target_player_id: Option<u32>` is serialized as `target_player: u32` in `UserData`. The code uses `unwrap_or(0)` to convert None to 0. If player ID 0 is ever assigned (it's not currently — IDs start at 1), or if a player legitimately targets ID 0, this becomes ambiguous.

**Fix:** Add a reserved sentinel value (e.g. `u32::MAX` for "no target") and document it in the protocol. Or add a separate `target_player: Option<u32>` field to the message that uses a presence byte.

---

## Style / Minor

### #11 Remove unused `states.rs`

**File:** `src/states.rs`

**Problem:** The `AppStates` enum is defined but never used anywhere in the codebase. The app uses dynamic dispatch via `Box<dyn Screen>` instead of an enum-based state machine. This file is dead code.

**Fix:** Delete `src/states.rs` and remove it from any module declarations.

---

### #12 `lib.rs` exposes `game` and `core` as public modules unnecessarily

**File:** `src/lib.rs:11-12`

**Problem:** `pub mod game` and `pub mod core` expose internal implementation details to consumers of the library. The only items that need to be public are `App`, `Screen`, and the screen implementations. External crates don't need to know about `NetworkMessage` internals or game object details.

**Fix:** Make these `pub(crate)` or `mod` (private), and only re-export the specific types that are actually part of the public API (e.g., `NetworkMessage` if needed by external code).

---

### #13 Magic numbers scattered throughout the codebase

**Files:** `src/game/objects/ship.rs`, `src/game/objects/bullet.rs`, `src/screens/host.rs`, `src/core/networking.rs`

**Problem:** Hardcoded values like `42069` (port), `100.0` (coordinate space), `1.2` (bullet lifetime), `60.0` (acceleration), `0.7` (friction) are buried in the code with no central configuration. Changing any of these requires hunting through multiple files.

**Fix:** Group game constants into a `config` module or `const` block at the top of each relevant file. For networking, extract the port to a named constant like `DEFAULT_PORT`.

---

### #14 `Start` screen doesn't validate hostname before navigation

**File:** `src/screens/start.rs:28-29`

**Problem:** The Play button fires `ScreenCommand::Play { hostname, username }` regardless of whether `hostname` is empty. The `Player` screen handles this by falling back to single-player mode, but there's no visual feedback to the user that their input was invalid or that the game is running locally.

**Fix:** Either disable the Play button when hostname is empty (with a tooltip like "Playing single-player"), or show a confirmation dialog. Add a `is_single_player` flag to `ScreenCommand::Play`.

---

## New TODOs

### #15 `GameEvent::PlayerTarget` needs network wiring for asteroid transfer

**File:** `src/game/game.rs:240`, `src/screens/player.rs:189-191`

**Problem:** Players will send asteroids to each other (see design doc). This requires a `NetworkMessage::AsteroidSent` / `AsteroidReceived` pair and corresponding `GameEvent` variants.

**Fix:** Add `GameEvent::AsteroidSent { id, size }` and `GameEvent::AsteroidReceived { asteroid }` variants. Wire them in the Player screen to send/receive `NetworkMessage::AsteroidSent`.

---

### #16 `UserData` target_player ambiguity with sentinel value 0

**File:** `src/core/networking.rs:37`, `src/screens/host.rs:242`

**Problem:** `target_player: u32` in `UserData` uses `unwrap_or(0)` to convert `None` to 0. Player ID 0 is reserved (IDs start at 1), but if a player targets ID 0 it becomes ambiguous with "no target".

**Fix:** Use `u32::MAX` as a sentinel for "no target" and document it in the protocol.

---

### #17 `NetworkMessage::Ready` uses `u8` instead of `bool`

**File:** `src/core/networking.rs:19`, `src/core/networking.rs:139`

**Problem:** `is_ready: u8` accepts any byte value (255, 42, etc.) not just 0 and 1.

**Fix:** Define a `ReadyState { Ready, NotReady }` enum and use it in the message type.
