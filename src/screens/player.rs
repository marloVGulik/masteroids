use crate::screen::ScreenCommand;
use crate::screen::Screen;
use crate::core::networking::{NetworkManager, NetworkMessage};
use crate::game::game::{Game, GameInput, GameState};

pub struct Player {
    game: Game,
    networkmanager: NetworkManager,
    hostname: String,
    username: String,
}

impl Player {
    pub fn new(hostname: String, username: String) -> Self {
        let newhn = hostname.clone() + ":42069";
        Self {
            game: Game::new(),
            networkmanager: NetworkManager::new("0.0.0.0:0"),
            hostname: newhn,
            username,
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

            // let painter = ui.painter();
            // let pos = egui::pos2(100.0, 100.0);
            // painter.circle_filled(pos, 5.0, egui::Color32::RED); // Draw an asteroid

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

            ui.label(format!("Collected rocks: {}", self.game.get_collected_rocks()));
        });

        ctx.request_repaint();

        cmd
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {
        self.game.activate();
        self.networkmanager.emit(&self.hostname, NetworkMessage::Connect { name: self.username.clone() });
    } 

    fn update(&mut self, ctx: &egui::Context, _frame: &eframe::Frame) {
        self.networkmanager.process_incoming(|_addr, msg| {
            match msg {
                NetworkMessage::StartGame => {
                    self.game.set_state(GameState::Active);
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
                    crate::game::game::GameEvent::Died => {
                        println!("Player died!");
                    },
                    crate::game::game::GameEvent::AsteroidDestroyed { size } => {
                        println!("Asteroid destroyed with size: {}", size);
                    },
                }
            });
        });
    }
}