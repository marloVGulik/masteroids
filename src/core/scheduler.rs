//! A generic task scheduler for timed, optionally repeating actions.
//!
//! Tasks are identified by name and can be scheduled, unscheduled, or have their wait time
//! updated at runtime. Non-repeating tasks are removed via `remove_fired()` after they fire.

use std::time::Duration;

pub struct Task<T> {
    name: String,
    start_time: std::time::Instant,
    wait_time: Duration,
    repeat: bool,
    has_ran: bool,
    action: T,
}

/// A scheduler that runs tasks at specified intervals.
///
/// Each task holds an action of type `T`. The `update` method calls a handler closure
/// for each task whose wait time has elapsed. Non-repeating tasks must be cleaned up
/// via `remove_fired()`; repeating tasks continue until explicitly unscheduled.
pub struct Scheduler<T> {
    tasks: Vec<Task<T>>,
}

impl<T> Scheduler<T> {
    /// Creates a new empty scheduler.
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Schedules a task with the given name, wait duration, and action.
    ///
    /// If `repeat` is true the task fires every `wait_time`. If false it fires once.
    pub fn schedule(&mut self, name: &str, wait_time: Duration, repeat: bool, action: T) {
        self.tasks.push(Task {
            name: name.to_string(),
            start_time: std::time::Instant::now(),
            wait_time,
            repeat,
            has_ran: false,
            action,
        });
    }

    /// Removes any task with the given name.
    pub fn unschedule(&mut self, name: &str) {
        self.tasks.retain(|task| task.name != name);
    }

    /// Changes the wait time of an existing task.
    pub fn set_wait_time(&mut self, name: &str, wait_time: Duration) {
        if let Some(task) = &mut self.tasks.iter_mut().find(|s| s.name == name) {
            task.wait_time = wait_time;
        }
    }

    /// Advances the scheduler and fires any ready tasks.
    ///
    /// The `handler` closure receives a mutable reference to each ready task's action.
    pub fn update(&mut self, mut handler: impl FnMut(&mut T)) {
        let now = std::time::Instant::now();
        for task in &mut self.tasks {
            if now.duration_since(task.start_time) >= task.wait_time {
                handler(&mut task.action);
                if task.repeat {
                    task.start_time = now;
                }
                task.has_ran = true;
            }
        }
    }

    /// Removes all non-repeating tasks that have already fired.
    ///
    /// Call this after `update()` to clean up one-shot tasks whose actions have completed.
    pub fn remove_fired(&mut self) {
        self.tasks.retain(|task| !(task.has_ran && !task.repeat));
    }
}
