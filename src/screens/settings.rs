use crate::screen::ScreenCommand;
use crate::screen::Screen;



pub struct Settings;

impl Settings {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for Settings {
    fn ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, _order: egui::Order) -> Option<ScreenCommand> {
        let mut cmd = None;

        ui.vertical_centered(|ui| {
            ui.heading("SETTINGS");
            if ui.button("Back").clicked() {
                cmd = Some(ScreenCommand::Start);
            }
        });

        cmd
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {

    }

    fn update(&mut self, _ctx: &egui::Context, _event: &eframe::Frame) -> Option<ScreenCommand> {
        return None;
    }
}