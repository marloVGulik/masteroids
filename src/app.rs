use egui;
use eframe;
use eframe::{App as EframeApp};

use crate::{Screen, screen, screens};


pub struct App {
    label: String,
    current_screen: Box<dyn Screen>
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "Masteroids".to_owned(),
            current_screen: Box::new(screens::start::Start::new()),
        }
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl EframeApp for App {    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut next_screen: Option<screen::ScreenCommand> = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.label);
            ui.label("Frietsaus moment");

            next_screen = self.current_screen.ui(ctx, ui, egui::Order::Foreground);
        });

        if let Some(cmd) = next_screen {
            match cmd {
                screen::ScreenCommand::Start => {
                    self.current_screen = Box::new(screens::start::Start::new());
                }
                screen::ScreenCommand::Settings => {
                    self.current_screen = Box::new(screens::settings::Settings::new());
                }
                screen::ScreenCommand::ExitProgram => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                _ => {
                    // NOT IMPLEMENTED
                }
            }
        }
    }

}