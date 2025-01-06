use core::panic;

use eframe::egui;
use futures::prelude::*;
use tarpc::context;
use tokio::sync::mpsc;

pub struct Application {
    name: String,
    age: u32,
    gui_rx: mpsc::Receiver<GuiMessage>,
}

impl Application {
    pub fn new(gui_rx: mpsc::Receiver<GuiMessage>) -> Self {
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
                GuiMessage::SendString(_) => {
                    panic!();
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
    async fn handle_recvfrom(data: Vec<u8>) -> String;
}
#[derive(Clone)]
pub struct HelloServer {
    pub gui_tx: mpsc::Sender<GuiMessage>,
}

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        println!("HelloServer hello imlpl");

        self.gui_tx
            .send(GuiMessage::SendString("SendString".to_string()))
            .await
            .expect("TODO");

        format!("Hello, {name}! You are connected")
    }

    async fn handle_recvfrom(self, _: context::Context, data: Vec<u8>) -> String {
        println!("handle_recvfrom: {data:?}");

        for (_, byte) in data.iter().enumerate() {
            println!("got {byte:02X}");
        }

        "got it".to_string()
    }
}

pub async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
