//! Shared types and utilities.
//!
//! This module contains:
//!
//! - `gameobject` — traits for game objects (`GameObject`, `Collidable`)
//! - `physics` — collision detection and response helpers
//! - `scheduler` — a generic task scheduler for timed events
//! - `networking` — UDP-based networking with message serialization

pub mod gameobject;
pub mod networking;
pub mod physics;
pub mod scheduler;
