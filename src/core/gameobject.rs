//! Core game object abstraction.
//!
//! `GameObject` defines the interface for anything that can be updated and drawn.
//! `Collidable` defines the interface for objects that support hitbox-based collision checks.

pub trait GameObject {
    /// Update this object by the given delta time.
    fn update(&mut self, dt: f64);
    /// Draw this object into the given egui UI.
    fn draw(&self, ui: &mut egui::Ui, size: f32, play_area: egui::Rect);
}

/// Trait for objects that can report whether a point falls within their collider.
pub trait Collidable {
    /// Returns true if the given screen-space point intersects this object.
    fn check_collision(&self, point: egui::Pos2) -> bool;
}
