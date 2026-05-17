use crate::screen::ScreenCommand;
use crate::screen::Screen;
use crate::core::networking::{NetworkManager, NetworkMessage};
use crate::game::game::{Game, GameInput, GameState, GameEvent};

pub struct OtherPlayer {
    name: String,
    health: u8,
    score: u32,
    id: u32,
    targets_you: bool
}
pub struct Player {
    game: Game,
    other_players: Vec<OtherPlayer>,
    networkmanager: NetworkManager,
    hostname: String,
    username: String,
    user_amount: u8,
    my_id: u32,
    is_ready: bool
}

impl Player {
    pub fn new(hostname: String, username: String) -> Self {
        // let newhn = hostname.clone() + ":42069";
        let new_hostname = hostname.clone();
        Self {
            game: Game::new(),
            other_players: vec!(),
            networkmanager: NetworkManager::new("0.0.0.0:0"),
            hostname: new_hostname,
            username,
            user_amount: 0,
            my_id: 0,
            is_ready: false
        }
    }
}

impl Screen for Player {
    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, _order: egui::Order) -> Option<ScreenCommand> {
        let mut cmd = None;

        ui.vertical_centered(|ui| {
            ui.heading("Game");
            if ui.button("Back").clicked() {
                cmd = Some(ScreenCommand::Start);
            }
            if self.game.get_state() == GameState::Waiting {
                let mut text = "Ready!";
                if self.is_ready {
                    text = "Not ready...";
                }
                if ui.button(text).clicked() {
                    self.is_ready = !self.is_ready;
                    let msg = NetworkMessage::Ready {
                        is_ready: self.is_ready as u8
                    };
                    self.networkmanager.emit(&self.hostname, &msg);
                }
            }

            // let painter = ui.painter();
            // let pos = egui::pos2(100.0, 100.0);
            // painter.circle_filled(pos, 5.0, egui::Color32::RED); // Draw an asteroid


            ui.label(format!("Connected players: {}", self.user_amount));

            let available_rect = ui.available_rect_before_wrap();
            let width = available_rect.width();
            let height = available_rect.height();
            let size = width.min(height);
            let play_area = egui::Rect::from_center_size(
                available_rect.center(), 
                egui::vec2(size, size)
            );
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.painter().rect_stroke( // View box
                    play_area, 
                    0.0, 
                    (2.0, egui::Color32::WHITE), 
                    egui::StrokeKind::Inside
                );
                let mut game_ui = ui.new_child(egui::UiBuilder::new().max_rect(play_area));
                
                self.game.draw(&mut game_ui, size, play_area);
            });
            ui.label(format!("Health: {}       Collected rocks: {}", self.game.get_health(), self.game.get_collected_rocks()));
        });

        ctx.request_repaint();

        cmd
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {
        self.game.activate();

        // If hostname is filled in:
        if !self.hostname.is_empty() {
            println!("Connecting to {}", self.hostname);
            self.networkmanager.emit(&self.hostname, &NetworkMessage::Connect { name: self.username.clone() });
        } else {
            self.game.set_state(GameState::Active);
        }
    } 

    fn update(&mut self, ctx: &egui::Context, _frame: &eframe::Frame) -> Option<ScreenCommand> {
        let mut cmd = None;

        self.networkmanager.process_incoming(|_addr, msg| {
            match msg {
                NetworkMessage::Reject { reason } => {
                    println!("Error connecting {}", reason);
                    cmd = Some(ScreenCommand::Start);
                },
                NetworkMessage::Accept { id } => {
                    self.my_id = *id;
                    println!("My id is {}", id);
                }
                NetworkMessage::StartGame => {
                    println!("Starting game!");
                    self.game.set_state(GameState::Active);
                },
                NetworkMessage::UserData { id, score, health, target_player: target_id, name } => {
                    println!("User data: {}:{}", id, name);
                    if let Some(user) = self.other_players.iter_mut().find(|p| p.id == *id) {
                        user.score = *score;
                        user.health = *health;
                        user.targets_you = *target_id == self.my_id;
                        user.name = name.to_string();
                    } else {
                        let new_user = OtherPlayer { name: name.to_string(), health: *health, score: *score, id: *id, targets_you: false };
                        self.other_players.push(new_user);
                    }
                },
                NetworkMessage::Alive => {
                    self.networkmanager.emit(&self.hostname, &NetworkMessage::Alive);
                },
                NetworkMessage::UserAmount { amount } => {
                    self.user_amount = *amount;
                },
                NetworkMessage::SummonAsteroid { x, y, direction, speed, size } => {
                    self.game.interact(GameInput::SummonAsteroid { x: *x, y: *y, direction: *direction, speed: *speed, size: *size });
                },
                _ => {}
            }
        });

        ctx.input(|i| {
            // Movement
            if i.key_down(egui::Key::W) {
                self.game.interact(GameInput::Forward { dt: i.stable_dt });
            }
            if i.key_down(egui::Key::A) {
                self.game.interact(GameInput::Left { dt: i.stable_dt });
            } 
            if i.key_down(egui::Key::D) {
                self.game.interact(GameInput::Right { dt: i.stable_dt });
            }

            // Shooting
            if i.key_down(egui::Key::Space) {
                self.game.interact(GameInput::Shoot { current_time: i.time });
            }

            self.game.update(i.stable_dt, i.time, |event| {
                match event {
                    GameEvent::Damage { health } => {
                        println!("Player got damaged! Health: {}", health);
                        if health == 0 {
                            cmd = Some(ScreenCommand::Start);
                        }
                        self.networkmanager.emit(&self.hostname, &NetworkMessage::PlayerDamaged);
                    },
                    GameEvent::AsteroidDestroyed { size } => {
                        println!("Asteroid destroyed with size: {}", size);

                        self.networkmanager.emit(&self.hostname, &NetworkMessage::AsteroidHit { size });
                    },
                    GameEvent::PlayerTarget { id } => {
                        println!("Player targeted with ID: {}", id);
                    },
                }
            });
        });
        
        return cmd;
    }
}