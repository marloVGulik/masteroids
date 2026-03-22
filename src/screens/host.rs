use crate::core::networking::{NetworkMessage, NetworkManager};
use crate::screen::ScreenCommand;
use crate::screen::Screen;
use crate::core::scheduler::{Scheduler};

use rand::{self, RngExt};

const USER_TIMEOUT: u8 = 10; // Number of alive checks before a user is considered disconnected

#[derive(Clone)]
pub struct Users {
    name: String,
    addr: std::net::SocketAddr,
    id: u32,
    score: u32,
    last_alive: u8,
    target_player_id: Option<u32>,
    is_ready: bool
}
impl Users {
    pub fn new(name: String, addr: std::net::SocketAddr, id: u32) -> Self {
        Self {
            name,
            addr,
            id,
            score: 0,
            last_alive: 0,
            target_player_id: None,
            is_ready: false,
        }
    }
}
enum Tasks {
    CheckAlive,
    SummonAsteroids,
}

pub struct Host {
    users: Vec<Users>,
    networkmanager: NetworkManager,
    scheduler: Scheduler<Tasks>,
    randomizer: rand::prelude::ThreadRng,
    current_user_id: u32,
}

impl Host {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            networkmanager: NetworkManager::new("[::]:42069"),
            scheduler: Scheduler::new(),
            randomizer: rand::rng(),
            current_user_id: 0,
        }
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
                    if self.users.iter().filter(|user| !user.is_ready).count() > 0 {
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
                    self.current_user_id = self.current_user_id + 1;
                    new_player_id = self.current_user_id;
                    new_player_name = name.clone();
                    
                    self.users.push(Users::new(new_player_name.clone(), addr, new_player_id));
                    println!("New user connected: {}", self.users.last().unwrap().name);
                    
                    update_player_amount = true;
                },
                NetworkMessage::Ready { is_ready } => {
                    if let Some(user) = self.users.iter_mut().find(|u| u.addr == addr) {
                        println!("User is_ready: {}", *is_ready);
                        user.is_ready = *is_ready > 0;
                    }

                    if self.users.iter().filter(|user| !user.is_ready).count() == 0 {
                        // println!("All players are ready");
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

                        //ToDo: Check if it is possible for the user to have hit an asteroid of that size (anti-cheat)
                    }
                },
                NetworkMessage::TargetPlayer { id } => {
                    if let Some(user) = self.users.iter_mut().find(|u| u.addr == addr) {
                        user.target_player_id = Some(id.clone());
                        println!("{} is targeting player with ID: {}", user.name, id);
                    }
                },
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
                self.emit_all(&NetworkMessage::ConnectShare { id: u.id, name: u.name.to_string() });
            }
        }

        if self.users.len() > 1 && start_game {
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