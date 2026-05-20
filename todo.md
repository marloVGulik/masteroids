# Masteroids Code Review TODOs

## Critical

### ~~#1~~ ~~Fix `Scheduler::update` retain logic silently drops one-shot tasks~~ ✅ Done

**File:** `src/core/scheduler.rs:50`

**Problem:** The retain at the end of `update()` deletes every non-repeating task immediately after it fires. Since `has_ran` is set inside the handler closure and the retain runs in the same call, the task is removed before it can ever fire again. This works accidentally for the current use case (immunity timer fires once and dies), but is a latent bug — any scheduler task that needs to fire exactly once and then be kept for inspection will be lost.

**Fix:** Remove the retain from `update()`, add explicit `remove_fired()` method, and call it after `update()` in `Game::update()`.

---

### #2 `Game::activate()` is a no-op — game never starts in single-player mode

**File:** `src/game/game.rs:69-77`

**Problem:** `activate()` contains only commented-out code. The only way to start the game is for the host to send `StartGame`, or for `Player::on_activate()` to call `game.set_state(GameState::Active)` directly. There is no way to play single-player from the Start screen (empty hostname) reliably.

**Fix:** Have `activate()` set `state = GameState::Active`, spawn initial asteroids, and emit a `GameEvent` if needed. Remove the commented-out code or move it to a config file.

---

### #3 Host liveness check is backwards — `last_alive` increments every frame regardless

**File:** `src/screens/host.rs:98-104`, `src/screens/host.rs:172-177`

**Problem:** The `CheckAlive` task increments `last_alive` for every user every second. But `last_alive` is only reset when the host *receives* an `Alive` message from that user (line 172). The problem: `NetworkManager::emit` sends to the *resolved target address*, but the host binds to `[::]:42069` and doesn't send its own Alive pings — it only *sends* them to users. Users are supposed to respond with their own `Alive` ping, but `Player::update` only sends `Alive` in response to receiving one (`NetworkMessage::Alive` → emit back). This creates a chicken-and-egg problem: if a user never receives a ping, they never send one, and their counter keeps growing. More importantly, the host itself never resets its own counter since it never receives a ping from itself.

**Fix:** Track `last_seen` separately — reset it when the host *receives* any message from a user (not just Alive). The `CheckAlive` task should only increment a timeout counter per user, resetting it on any inbound packet.

---

## High

### #4 `NetworkManager::resolver` silently swallows DNS errors

**File:** `src/core/networking.rs:213-227`

**Problem:** `to_socket_addrs()` returns a `Result<IntoIter>`. The code uses `if let Ok(...)` but if resolution fails, it returns `None` and the caller (`emit`) silently drops the message. There's no logging, no error return, and no fallback. A typo in the hostname just silently fails to send.

**Fix:** Return `Result<SocketAddr>` instead of `Option`, or at least log a warning on failure. Consider accepting `SocketAddr` directly as an alternative constructor.

---

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
