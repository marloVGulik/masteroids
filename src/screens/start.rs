use crate::screen::ScreenCommand;
use crate::screen::Screen;



pub struct Start {
    hostname: String,
    username: String,
}

impl Start {
    pub fn new() -> Self {
        Self {
            hostname: "".to_string(),
            username: "".to_string(),
        }
    }
}

impl Screen for Start {
    fn ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, _order: egui::Order) -> Option<ScreenCommand> {
        let mut cmd = None;

        ui.vertical_centered(|ui| {
            ui.heading("MASTEROIDS");
            ui.text_edit_singleline(&mut self.hostname);
            ui.text_edit_singleline(&mut self.username);
            if ui.button("Play").clicked() {
                cmd = Some(ScreenCommand::Play { hostname: self.hostname.clone(), username: self.username.clone() });
            }
            if ui.button("Host").clicked() {
                cmd = Some(ScreenCommand::Host);
            }
            if ui.button("Settings").clicked() {
                cmd = Some(ScreenCommand::Settings);
            }
            if ui.button("Exit :(").clicked() {
                cmd = Some(ScreenCommand::ExitProgram);
            }
            ui.label("controls");
            ui.label("forwards: w");
            ui.label("turn left: a");
            ui.label("turn right: d");
            ui.label("shoot: space");
        });

        cmd
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {

    }

    fn update(&mut self, _ctx: &egui::Context, _event: &eframe::Frame) -> Option<ScreenCommand> {
        return None;
    }
}
