use std::{
    sync::mpsc::{channel, Receiver, TryRecvError},
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
            rx,
        }
    }
}

// TODO: This doesn't work as-is, look at https://github.com/emilk/egui/discussions/484
// to see how it's actually done.
impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle mspc channel
        loop {
            match self.rx.try_recv() {
                Ok(msg) => match msg {
                    GuiMessage::Hello(_) => {
                        self.age += 1;
                    }
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    println!("Channel disconnected");
                    break;
                }
            }
        }

        // Handle UI
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
    let mut n = 100;
    thread::spawn(move || loop {
        tx.send(GuiMessage::Hello("World!".to_string())).unwrap();
        thread::sleep(Duration::from_millis(100));
        n -= 1;

        if n < 0 {
            break;
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    let app = Application::new(rx);
    eframe::run_native("My egui App", options, Box::new(|_cc| Ok(Box::new(app))))
}
