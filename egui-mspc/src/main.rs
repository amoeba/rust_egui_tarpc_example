use std::{
    sync::mpsc::{channel, Receiver},
    thread,
    time::Duration,
};

use eframe::egui;

pub enum GuiMessage {
    Hello(String),
}

pub struct Application {
    name: String,
    age: u32,
    rx: Receiver<GuiMessage>,
}

impl Application {
    pub fn new(rx: Receiver<GuiMessage>) -> Self {
        Self {
            name: "Test".to_string(),
            age: 40,
            rx: rx,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                GuiMessage::Hello(msg) => {
                    println!("Received message in GUI: {}", msg);
                    self.age += 1;
                }
            }
        }

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

    let (tx, rx): (std::sync::mpsc::Sender<GuiMessage>, Receiver<GuiMessage>) = channel();

    let tx = tx.clone();
    thread::spawn(move || loop {
        tx.send(GuiMessage::Hello("World!".to_string())).unwrap();
        thread::sleep(Duration::from_secs(1));
    });

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::new(Application::new(rx)))),
    )
}
