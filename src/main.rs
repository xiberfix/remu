use eframe::egui;

use remu::App;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        ..Default::default()
    };

    eframe::run_native(
        "remu",
        options,
        Box::new(|_cc| Box::new(App::default())),
    )
}
