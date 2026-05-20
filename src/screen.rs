//! Screen navigation trait and commands.
//!
//! Every game screen (start, settings, player, host) implements the `Screen` trait.
//! The screen's `update()` and `ui()` methods each return an optional `ScreenCommand`
/// that tells `App` which screen to navigate to next.

use egui::{Context, Ui};

/// A command to switch screens or exit the application.
pub enum ScreenCommand {
    /// Navigate to the start (main menu) screen.
    Start,
    /// Navigate to the player screen to join a game.
    Play {
        /// Hostname or IP of the host server.
        hostname: String,
        /// Display name of the player.
        username: String,
    },
    /// Navigate to the host screen to create a new game.
    Host,
    /// Navigate to the settings screen.
    Settings,
    /// Close the application.
    ExitProgram,
}

/// Trait implemented by all game screens.
///
/// The egui screen pattern: each screen manages its own state, renders UI via `ui()`,
/// runs per-frame logic via `update()`, and receives an activation callback via
/// `on_activate()`.
pub trait Screen {
    /// Called when this screen becomes the active screen.
    fn on_activate(&mut self, ctx: &Context);

    /// Per-frame update logic (runs before the egui frame).
    ///
    /// Returns a `ScreenCommand` if the screen wants to navigate away, `None` otherwise.
    fn update(&mut self, ctx: &Context, event: &eframe::Frame) -> Option<ScreenCommand>;

    /// Render the screen's egui UI.
    ///
    /// Returns a `ScreenCommand` if the screen wants to navigate away, `None` otherwise.
    fn ui(&mut self, ctx: &Context, ui: &mut Ui, order: egui::Order) -> Option<ScreenCommand>;
}
