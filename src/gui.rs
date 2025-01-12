use std::sync::Arc;

use eframe::egui;
use tokio::sync::{
    mpsc::{error::TryRecvError, Receiver},
    Mutex,
};

use crate::rpc::GuiMessage;

pub struct Application {
    string: String,
    gui_rx: Arc<Mutex<Receiver<GuiMessage>>>,
}

impl Application {
    pub fn new(gui_rx: Arc<Mutex<Receiver<GuiMessage>>>) -> Self {
        Self {
            string: "Unset".to_string(),
            gui_rx,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle mspc channel
        loop {
            match self.gui_rx.try_lock().unwrap().try_recv() {
                Ok(msg) => match msg {
                    GuiMessage::Hello(_) => {
                        println!("GUI got Hello");
                    }
                    GuiMessage::UpdateString(value) => {
                      println!("GUI got UpdateString with value {value}");
                      self.string = value.to_string();
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
            ui.heading("Debugging");
            ui.horizontal(|ui| {
                let string_label = ui.label("String: ");
                ui.text_edit_singleline(&mut self.string)
                    .labelled_by(string_label.id);
            });
        });
    }
}
