use eframe::egui;
use futures::prelude::*;
use tarpc::context;
use tokio::sync::mpsc;

pub struct Application {
    name: String,
    age: u32,
    gui_rx: mpsc::UnboundedReceiver<ServerMessage>,
}

impl Application {
    pub fn new(gui_rx: mpsc::UnboundedReceiver<ServerMessage>) -> Self {
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
                ServerMessage::NewData(_) => {
                    println!("TODO");
                }
                ServerMessage::StatusUpdate(_) => {
                    println!("TODO");
                }
                ServerMessage::Error(_) => {
                    println!("TODO");
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

// Message from Server to GUI
pub enum ServerMessage {
    NewData(String),
    StatusUpdate(String),
    Error(String),
}

// Messages from GUI to RPC server
pub enum GuiMessage {
    SendData(String),
    RequestUpdate,
}

#[tarpc::service]
pub trait World {
    async fn hello(name: String) -> String;
    async fn handle_recvfrom(data: Vec<u8>) -> String;
}
#[derive(Clone)]
pub struct HelloServer {
    pub gui_tx: mpsc::UnboundedSender<ServerMessage>,
}

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        self.gui_tx
            .send(ServerMessage::NewData("got hello".to_string()))
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
