use eframe::egui;

pub struct App {
    path_text: String,
    address_text: String,
    error: Option<String>,
    memory: Vec<u8>,
}


impl Default for App {
    fn default() -> Self {
        Self {
            path_text: "".to_owned(),
            address_text: "".to_owned(),
            error: None,
            memory: vec![0; 0x10000], // 64 KiB
        }
    }
}


impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::Window::new("Loader")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    self.loader_view(ui);
                });
        });
    }
}


impl App {
    fn loader_view(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("loader")
            .num_columns(2)
            .spacing([24.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                let Self { path_text: file, address_text: address, .. } = self;

                ui.label("File:");
                ui.add(egui::TextEdit::singleline(file).hint_text("path"));
                ui.end_row();

                ui.label("Address:");
                ui.add(egui::TextEdit::singleline(address).hint_text("xxxx"));
                ui.end_row();

                if ui.button("Load").clicked() {
                    self.load();
                };

                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::RED, error);
                }
                ui.end_row();
            });
    }


    fn load_data(&mut self) -> Result<(), String> {
        // parse address
        let begin = usize::from_str_radix(&self.address_text.to_owned(), 16)
            .map_err(|e| e.to_string())?;

        // read file
        let data = std::fs::read(&self.path_text)
            .map_err(|e| e.to_string())?;

        // copy data into memory
        let end = begin + data.len();
        let len = self.memory.len();
        if end > len {
            return Err("address out of range".to_owned());
        }
        self.memory[begin..end].copy_from_slice(&data);

        Ok(())
    }


    fn load(&mut self) {
        self.error = self.load_data().err();
    }
}
