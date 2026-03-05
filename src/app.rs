use egui;
use eframe;
use eframe::{App as EframeApp};

pub struct App {
    label: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "Masteroids".to_owned(),
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.label);
            ui.label("Frietsaus moment");
            
        });
    }

}