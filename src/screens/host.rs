use crate::core::networking::{NetworkMessage, NetworkManager};
use crate::screen::ScreenCommand;
use crate::screen::Screen;
use crate::core::scheduler::{Scheduler};

const USER_TIMEOUT: u8 = 10; // Number of alive checks before a user is considered disconnected

#[derive(Clone)]
pub struct Users {
    name: String,
    addr: std::net::SocketAddr,
    id: u32,
    score: u32,
    last_alive: u8,
    target_player_id: Option<u32>,
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
        }
    }
}
enum Tasks {
    CheckAlive,
}

pub struct Host {
    users: Vec<Users>,
    networkmanager: NetworkManager,
    scheduler: Scheduler<Tasks>,
}

impl Host {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            networkmanager: NetworkManager::new("[::]:42069"),
            scheduler: Scheduler::new(),
        }
    }


    fn emit_all(&mut self, msg: &NetworkMessage) {
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
            }
        });

        // Process incoming network messages
        let mut update_player_amount = false;
        self.networkmanager.process_incoming(|addr, msg| {
            match msg {
                NetworkMessage::Connect { name } => {
                    let id = self.users.len() as u32 + 1; // Simple ID assignment
                    self.users.push(Users::new(name.clone(), addr, id));
                    println!("New user connected: {}", self.users.last().unwrap().name);
                    
                    update_player_amount = true;
                },
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
        }

        return None;
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {
        self.scheduler.schedule(
            "alive_check".to_string(), 
            std::time::Duration::from_secs(1), 
            true,
            Tasks::CheckAlive
        );
    }
}