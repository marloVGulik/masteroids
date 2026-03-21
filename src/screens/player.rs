use crate::screen::ScreenCommand;
use crate::screen::Screen;
use crate::core::networking::{NetworkManager, NetworkMessage};
use crate::game::game::{Game, GameInput, GameState, GameEvent};

pub struct Player {
    game: Game,
    networkmanager: NetworkManager,
    hostname: String,
    username: String,
    user_amount: u8,
    is_ready: bool
}

impl Player {
    pub fn new(hostname: String, username: String) -> Self {
        // let newhn = hostname.clone() + ":42069";
        let new_hostname = hostname.clone();
        Self {
            game: Game::new(),
            networkmanager: NetworkManager::new("0.0.0.0:0"),
            hostname: new_hostname,
            username,
            user_amount: 0,
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
            if ui.button("Ready!").clicked() {
                self.is_ready = !self.is_ready;
                let msg = NetworkMessage::Ready {
                    is_ready: self.is_ready as u8
                };
                self.networkmanager.emit(&self.hostname, &msg);
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
            ui.painter().rect_stroke( // View box
                play_area, 
                0.0, 
                (2.0, egui::Color32::WHITE), 
                egui::StrokeKind::Inside
            );
            let mut game_ui = ui.new_child(egui::UiBuilder::new().max_rect(play_area));
            
            self.game.draw(&mut game_ui, size, play_area);

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
                NetworkMessage::StartGame => {
                    println!("Starting game!");
                    self.game.set_state(GameState::Active);
                },
                NetworkMessage::Alive => {
                    self.networkmanager.emit(&self.hostname, &NetworkMessage::Alive);
                },
                NetworkMessage::UserAmount { amount } => {
                    self.user_amount = *amount;
                }
                NetworkMessage::SummonAsteroid { x, y, direction, speed, size } => {
                    self.game.interact(GameInput::SummonAsteroid { x: *x, y: *y, direction: *direction, speed: *speed, size: *size });
                }
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