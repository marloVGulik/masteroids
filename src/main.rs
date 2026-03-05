use masteroids::App;


fn main() -> eframe::Result<()> { 
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Masteroids",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
