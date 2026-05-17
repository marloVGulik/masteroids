use crate::core::networking::{NetworkMessage, NetworkManager};
use crate::screen::ScreenCommand;
use crate::screen::Screen;
use crate::core::scheduler::{Scheduler};

use rand::{self, RngExt};

const USER_TIMEOUT: u8 = 10; // Number of alive checks before a user is considered disconnected

#[derive(Clone)]
pub struct User {
    name: String,
    addr: std::net::SocketAddr,
    id: u32,
    score: u32,
    health: u8,
    last_alive: u8,
    target_player_id: Option<u32>,
    is_ready: bool,
    is_updated: bool
}
impl User {
    pub fn new(name: String, addr: std::net::SocketAddr, id: u32) -> Self {
        Self {
            name,
            addr,
            id,
            score: 0,
            health: 3,
            last_alive: 0,
            target_player_id: None,
            is_ready: false,
            is_updated: false
        }
    }
}
enum Tasks {
    CheckAlive,
    SummonAsteroids,
}

pub struct Host {
    users: Vec<User>,
    networkmanager: NetworkManager,
    scheduler: Scheduler<Tasks>,
    randomizer: rand::prelude::ThreadRng,
    current_user_id: u32,
    started: bool
}

impl Host {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            networkmanager: NetworkManager::new("[::]:42069"),
            scheduler: Scheduler::new(),
            randomizer: rand::rng(),
            current_user_id: 0,
            started: false
        }
    }

    fn _get_user_by_id(&self, id: u32) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
    }

    fn emit_all(&self, msg: &NetworkMessage) {
        for user in &self.users {
            self.networkmanager.emit(&user.addr.to_string(), msg);
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
    fn update(&mut self, _ctx: &egui::Context, _event: &eframe::Frame) -> Option<ScreenCommand> {
        // Check scheduled tasks
        self.scheduler.update(|tasks| {
            match tasks {
                Tasks::CheckAlive => {
                    for user in &mut self.users {
                        user.last_alive += 1;
                        self.networkmanager.emit(&user.addr.to_string(), &NetworkMessage::Alive);
                    }
                },
                Tasks::SummonAsteroids => {
                    // Old starting logic
                    // if self.users.iter().filter(|user| !user.is_ready).count() > 0 {
                    //     return;
                    // }
                    if self.started == false {
                        return;
                    }

                    // println!("Time for new asteroids for {} users", self.users.len());
                    for user in self.users.iter() {
                        let x: f32 = self.randomizer.random_range(0.0..=100.0);
                        let y: f32 = self.randomizer.random_range(0.0..=100.0);
                        let direction: f32 = self.randomizer.random_range(0.0..=360.0);
                        let speed: f32 = self.randomizer.random_range(0.0..10.0);
                        let size: u8 = self.randomizer.random_range(0..=3);

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
        let mut new_player_id = 0;
        let mut new_player_name = String::new();
        let mut start_game = false;
        self.networkmanager.process_incoming(|addr, msg| {
            match msg {
                NetworkMessage::Connect { name } => {
                    // Reject if game has already started
                    if self.started {
                        self.networkmanager.emit_socket(&addr, &NetworkMessage::Reject { reason: 1 });
                        return;
                    }

                    self.current_user_id = self.current_user_id + 1;
                    new_player_id = self.current_user_id;
                    new_player_name = name.clone();
                    
                    self.users.push(User::new(new_player_name.clone(), addr, new_player_id));
                    println!("New user connected: {}", self.users.last().unwrap().name);
                    
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
                    if let Some(user) = self.users.iter_mut().find(|u| u.addr == addr) {
                        user.last_alive = 0; // Reset alive counter
                        // println!("Received alive from {}", user.name);
                    }
                },
                NetworkMessage::AsteroidHit { size: _ } => {
                    if let Some(user) = self.users.iter_mut().find(|u| u.addr == addr) {
                        user.score += 1; // Increase score based on asteroid size
                        user.is_updated = true;

                        //ToDo: Check if it is possible for the user to have hit an asteroid of that size (anti-cheat)
                    }
                },
                NetworkMessage::TargetPlayer { id } => {
                    if let Some(user) = self.users.iter_mut().find(|u| u.addr == addr) {
                        user.target_player_id = Some(id.clone());
                        println!("{} is targeting player: {}", user.id, id);
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

        // Remove dead users
        let old_size = self.users.len();
        self.users.retain(|user| user.last_alive < USER_TIMEOUT);
        if old_size != self.users.len() {
            update_player_amount = true;
        }

        // Emit user amount
        if update_player_amount {
            self.emit_all(&NetworkMessage::UserAmount { amount: self.users.len() as u8 });
            // self.emit_all(&NetworkMessage::ConnectShare { id: new_player_id, name: new_player_name });
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