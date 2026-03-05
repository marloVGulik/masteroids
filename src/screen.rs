use egui::{Context, Ui};

pub enum ScreenCommand {
    Start,
    Play,
    Settings,
    ExitProgram,
}
pub trait Screen {
    fn on_activate(&mut self, ctx: &Context);

    fn update(&mut self, ctx: &Context, event: &egui::RawInput);

    fn ui(&mut self, ctx: &Context, ui: &mut Ui, order: egui::Order) -> Option<ScreenCommand>;
}
