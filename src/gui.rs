use std::sync::Arc;

use eframe::egui::{self, ScrollArea, TextStyle};
use tokio::sync::{
    mpsc::{error::TryRecvError, Receiver},
    Mutex,
};

use crate::rpc::GuiMessage;

pub struct Application {
    string: String,
    logs: Vec<String>,
    gui_rx: Arc<Mutex<Receiver<GuiMessage>>>,
}

impl Application {
    pub fn new(gui_rx: Arc<Mutex<Receiver<GuiMessage>>>) -> Self {
        Self {
            string: "Unset".to_string(),
            logs: vec![],
            gui_rx,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle channel
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
                    GuiMessage::AppendLog(value) => {
                        println!("GUI got AppendLog with value {value}");
                        self.logs.push(value);
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

            let text_style = TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);
            let total_rows = self.logs.len();

            ui.vertical(|ui| {
                ScrollArea::vertical().auto_shrink(false).show_rows(
                    ui,
                    row_height,
                    total_rows,
                    |ui, row_range| {
                        for row in row_range {
                            let text = format!("{}", self.logs[row]);
                            ui.label(text);
                        }
                    },
                );
            });
        });
    }
}
