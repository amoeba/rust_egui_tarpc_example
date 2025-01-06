use eframe::egui;
use std::{future::Future, sync::mpsc as std_mpsc};
use tarpc::context;

pub struct Application {
    name: String,
    age: u32,
    gui_rx: std_mpsc::Receiver<GuiMessage>,
}

impl Application {
    pub fn new(gui_rx: std_mpsc::Receiver<GuiMessage>) -> Self {
        Self {
            name: "Test".to_string(),
            age: 40,
            gui_rx,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.gui_rx.try_recv() {
            match message {
                GuiMessage::SendString(msg) => {
                    println!("Received message in GUI: {}", msg);
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

pub enum GuiMessage {
    SendString(String),
}

#[tarpc::service]
pub trait World {
    async fn hello(name: String) -> String;
}

#[derive(Clone)]
pub struct HelloServer {
    pub gui_tx: std_mpsc::Sender<GuiMessage>,
}

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        println!("HelloServer hello impl");

        self.gui_tx
            .send(GuiMessage::SendString("SendString".to_string()))
            .expect("Failed to send message to GUI");

        format!("Hello, {name}! You are connected")
    }
}

pub async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
