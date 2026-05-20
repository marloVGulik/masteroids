//! The host screen: manages multiplayer game sessions.

use crate::core::networking::{NetworkMessage, NetworkManager};
use crate::screen::ScreenCommand;
use crate::screen::Screen;
use crate::core::scheduler::Scheduler;

use rand::{self, RngExt};

/// Number of consecutive missed packets before a user is considered disconnected.
const USER_TIMEOUT: u8 = 10;

/// A connected multiplayer user.
#[derive(Clone)]
pub struct User {
    /// Display name.
    pub name: String,
    /// Network address.
    pub addr: std::net::SocketAddr,
    /// Unique ID assigned by the host.
    pub id: u32,
    /// Current score.
    pub score: u32,
    /// Current health.
    pub health: u8,
    /// Last time a packet was received from this user (incremented by scheduler).
    pub last_seen: u8,
    /// ID of the player this user is targeting.
    pub target_player_id: Option<u32>,
    /// Whether this user has pressed ready.
    pub is_ready: bool,
    /// Whether this user's data has been emitted this frame.
    pub is_updated: bool,
}

impl User {
    /// Creates a new user with zero score and health.
    pub fn new(name: String, addr: std::net::SocketAddr, id: u32) -> Self {
        Self {
            name,
            addr,
            id,
            score: 0,
            health: 3,
            last_seen: 0,
            target_player_id: None,
            is_ready: false,
            is_updated: false,
        }
    }
}

/// Scheduled tasks managed by the host's scheduler.
enum Tasks {
    /// Periodically increments the last_seen counter for all users.
    CheckAlive,
    /// Spawns new asteroids for all connected players.
    SummonAsteroids,
}

/// The host game screen.
///
/// Manages connected users, processes incoming network messages, runs scheduled tasks
/// (alive checks, asteroid spawning), and broadcasts game state to all clients.
/// When all users are ready and there are 2+ players, the game starts automatically.
pub struct Host {
    users: Vec<User>,
    networkmanager: NetworkManager,
    scheduler: Scheduler<Tasks>,
    randomizer: rand::prelude::ThreadRng,
    current_user_id: u32,
    started: bool,
    pressure_enabled: bool,
}

impl Host {
    /// Creates a new host listening on `[::]:42069`.
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            networkmanager: NetworkManager::new("[::]:42069"),
            scheduler: Scheduler::new(),
            randomizer: rand::rng(),
            current_user_id: 0,
            started: false,
            pressure_enabled: true,
        }
    }

    fn _get_user_by_id(&self, id: u32) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
    }

    /// Emits a message to all connected users.
    fn emit_all(&self, msg: &NetworkMessage) {
        for user in &self.users {
            self.networkmanager.emit_socket(&user.addr, msg);
        }
    }
}

impl Screen for Host {
    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, _order: egui::Order) -> Option<ScreenCommand> {
        let mut cmd = None;

        ui.vertical_centered(|ui| {
            ui.heading("Host Game");
            if ui.button("Back").clicked() {
                cmd = Some(ScreenCommand::Start);
            }
            if ui.button(if self.pressure_enabled { "Disable Pressure" } else { "Enable Pressure" }).clicked() {
                self.pressure_enabled = !self.pressure_enabled;
            }

            ui.horizontal_centered(|user_ui| {
                for user in &self.users {
                    user_ui.label(format!("{} (ID: {})", user.name, user.id));
                    user_ui.label(format!("Score: {}", user.score));
                }
            });
        });

        ctx.request_repaint();

        cmd
    }

    /// Processes scheduler tasks and incoming network messages.
    ///
    /// This is the main game-loop host logic: it handles user connections, ready states,
    /// liveness checks, score updates, asteroid spawning, and game start conditions.
    fn update(&mut self, _ctx: &egui::Context, _event: &eframe::Frame) -> Option<ScreenCommand> {
        // Check scheduled tasks
        self.scheduler.update(|tasks| {
            match tasks {
                Tasks::CheckAlive => {
                    for user in &mut self.users {
                        user.last_seen += 1;
                    }
                },
                Tasks::SummonAsteroids => {
                    if self.started == false || !self.pressure_enabled {
                        return;
                    }

                    for user in self.users.iter() {
                        let x: f32 = self.randomizer.random_range(0.0..=100.0);
                        let y: f32 = self.randomizer.random_range(0.0..=100.0);
                        let direction: f32 = self.randomizer.random_range(0.0..=360.0);
                        let speed: f32 = self.randomizer.random_range(0.0..10.0);
                        let size: u8 = self.randomizer.random_range(1..=3);

                        self.networkmanager.emit_socket(
                            &user.addr, 
                            &NetworkMessage::SummonAsteroid { 
                                x, 
                                y, 
                                direction, 
                                speed,
                                size 
                            }
                        );
                    }
                }
            }
        });

        // Process incoming network messages
        let mut update_player_amount = false;
        let mut start_game = false;
        self.networkmanager.process_incoming(|addr, msg| {
            // Reset last_seen on any inbound message from a known user
            if let Some(user) = self.users.iter_mut().find(|u| u.addr == addr) {
                user.last_seen = 0;
            }

            match msg {
                NetworkMessage::Connect { name } => {
                    if self.started {
                        self.networkmanager.emit_socket(&addr, &NetworkMessage::Reject { reason: 1 });
                        return;
                    }

                    self.current_user_id = self.current_user_id + 1;
                    let player_id = self.current_user_id;
                    
                    self.users.push(User::new(name.clone(), addr, player_id));
                    println!("New user connected: {}", name);
                    
                    // Send Accept message directly to the new player with their ID
                    self.networkmanager.emit_socket(&addr, &NetworkMessage::Accept { id: player_id });
                    
                    update_player_amount = true;
                },
                NetworkMessage::Ready { is_ready } => {
                    if let Some(user) = self.users.iter_mut().find(|u| u.addr == addr) {
                        println!("User is_ready: {}", *is_ready);
                        user.is_ready = *is_ready > 0;
                    }

                    if self.users.iter().filter(|user| !user.is_ready).count() == 0 {
                        println!("All players are ready");
                        start_game = true;
                    }
                }
                NetworkMessage::Alive => {
                    // Heartbeat already handled above by resetting last_seen
                },
               NetworkMessage::AsteroidHit { size: _ } => {
                    if let Some(user) = self.users.iter_mut().find(|u| u.addr == addr) {
                        user.score += 1;
                        user.is_updated = true;
                    }
                },
                NetworkMessage::TargetPlayer { id } => {
                    if let Some(user) = self.users.iter_mut().find(|u| u.addr == addr) {
                        user.target_player_id = Some(*id);
                        println!("Player {} is targeting player: {}", user.id, id);
                    }
                },
                NetworkMessage::AttackPlayer { amount } => {
                    if let Some(user) = self.users.iter_mut().find(|u| u.addr == addr) {
                        if let Some(targeted_user_id) = user.target_player_id {
                            if let Some(targeted_user) = self.users.iter_mut().find(|u| u.id == targeted_user_id) {
                                self.networkmanager.emit_socket(&targeted_user.addr, &NetworkMessage::AttackPlayer { amount: *amount });
                            }
                        }
                    }
                }
                _ => {}
            }
        });

        // Remove dead users (and their asteroids are simply discarded — no redistribution)
        let old_size = self.users.len();
        self.users.retain(|user| user.last_seen < USER_TIMEOUT);
        if old_size != self.users.len() {
            update_player_amount = true;
        }

        // Emit user amount
        if update_player_amount {
            self.emit_all(&NetworkMessage::UserAmount { amount: self.users.len() as u8 });
            for u in self.users.iter() {
                self.emit_all(&NetworkMessage::UserData { 
                    id: u.id, 
                    score: u.score, 
                    health: u.health, 
                    target_player: u.target_player_id.unwrap_or(0), 
                    name: u.name.to_string() 
                });
            }
        }

        // Emit changed player data (self)
        let mut updated_user_ids: Vec<u32> = vec!();
        self.users.iter().filter(|u| u.is_updated == true).for_each(|u| {
            if let Some(target_id) = u.target_player_id {
                updated_user_ids.push(target_id);
            }

            self.emit_all(&NetworkMessage::UserData { 
                id: u.id, 
                score: u.score, 
                health: u.health, 
                target_player: u.target_player_id.unwrap_or(0), 
                name: u.name.to_string()
            });
        });
        self.users.iter_mut().filter(|u| updated_user_ids.iter().find(|su| **su == u.id).is_some()).for_each(|u| {
            u.is_updated = false;
        });

        if self.users.len() > 1 && start_game {
            println!("Starting game!");
            self.started = true;
            self.emit_all(&NetworkMessage::StartGame);
        }

        return None;
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {
        self.scheduler.schedule(
            "alive_check", 
            std::time::Duration::from_secs(1), 
            true,
            Tasks::CheckAlive
        );
        self.scheduler.schedule(
            "spawn_asteroid", 
            std::time::Duration::from_secs(5), 
            true, 
            Tasks::SummonAsteroids
        );
    }
}
