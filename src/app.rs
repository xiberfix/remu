use eframe::egui;

pub struct App {
    text: String,
    number: u32,
}


impl Default for App {
    fn default() -> Self {
        Self {
            text: "Hello, World!".to_owned(),
            number: 42,
        }
    }
}


impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { text, number } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My Application");
            ui.horizontal(|ui| {
                let label = ui.label("text: ");
                ui.text_edit_singleline(text).labelled_by(label.id);
            });
            ui.add(egui::Slider::new(number, 0..=100).text("number"));
            if ui.button("increment").clicked() {
                *number += 1;
            }
            ui.label(format!("text={}; number={}", text, number));
        });
    }
}
