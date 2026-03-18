use egui::{Context, Ui};

pub enum ScreenCommand {
    Start,
    Play { hostname: String, username: String },
    Host,
    Settings,
    ExitProgram,
}
pub trait Screen {
    fn on_activate(&mut self, ctx: &Context);

    fn update(&mut self, ctx: &Context, event: &eframe::Frame) -> Option<ScreenCommand>;

    fn ui(&mut self, ctx: &Context, ui: &mut Ui, order: egui::Order) -> Option<ScreenCommand>;
}
