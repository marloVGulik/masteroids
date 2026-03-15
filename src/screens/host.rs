use crate::core::networking::{NetworkMessage, NetworkManager};
use crate::screen::ScreenCommand;
use crate::screen::Screen;

pub struct Users {
    name: String,
    addr: std::net::SocketAddr,
    id: u32,
    score: u32,
    target_player_id: Option<u32>,
}
impl Users {
    pub fn new(name: String, addr: std::net::SocketAddr, id: u32) -> Self {
        Self {
            name,
            addr,
            id,
            score: 0,
            target_player_id: None,
        }
    }
}

pub struct Host {
    users: Vec<Users>,
    networkmanager: NetworkManager,
}

impl Host {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            networkmanager: NetworkManager::new("0.0.0.0:42069")
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
        });

        ctx.request_repaint();

        cmd
    }
    fn update(&mut self, _ctx: &egui::Context, _event: &eframe::Frame) {
        self.networkmanager.process_incoming(|addr, msg| {
            match msg {
                NetworkMessage::Connect { name } => {
                    let id = self.users.len() as u32 + 1; // Simple ID assignment
                    self.users.push(Users::new(name, addr, id));
                    println!("New user connected: {}", self.users.last().unwrap().name);
                },
                _ => {}
            }
        });
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {

    }
}