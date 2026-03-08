use crate::screen::ScreenCommand;
use crate::screen::Screen;



pub struct Start;

impl Start {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for Start {
    fn ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, _order: egui::Order) -> Option<ScreenCommand> {
        let mut cmd = None;

        ui.vertical_centered(|ui| {
            ui.heading("MASTEROIDS");
            if ui.button("Play").clicked() {
                cmd = Some(ScreenCommand::Play);
            }
            if ui.button("Settings").clicked() {
                cmd = Some(ScreenCommand::Settings);
            }
            if ui.button("Exit :(").clicked() {
                cmd = Some(ScreenCommand::ExitProgram);
            }
        });

        cmd
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {

    }

    fn update(&mut self, _ctx: &egui::Context, _event: &eframe::Frame) {
        
    }
}