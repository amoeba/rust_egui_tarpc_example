use eframe::egui;

pub struct Application {
    name: String,
    age: u32,
}

impl Application {
    pub fn new() -> Self {
        Self {
            name: "Test".to_string(),
            age: 40,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::new(Application::new()))),
    )
}
