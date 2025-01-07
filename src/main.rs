use core::panic;
use log::{debug, error, info, log_enabled, Level};
use std::{
    sync::{
        mpsc::{channel, Receiver, TryRecvError},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use eframe::egui;

pub enum GuiMessage {
    Hello(String),
}

pub enum PaintMessage {
    RequestRepaint,
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

    // Channel for sending updates related to mutating the GUI state
    let (tx, rx): (std::sync::mpsc::Sender<GuiMessage>, Receiver<GuiMessage>) = channel();

    // Create a second mpsc channel for sending updates from outside the
    // application's CreationContext into it.
    // TODO: I'm not sure if this actually works or is achieveable
    let (paint_tx, paint_rx): (
        std::sync::mpsc::Sender<PaintMessage>,
        Receiver<PaintMessage>,
    ) = channel();

    // WIP
    let final_paint_rx = Arc::new(Mutex::new(paint_rx));

    // WIP. Replace this with RPC code eventually.
    let tx = tx.clone();
    let ptx = paint_tx.clone();
    let mut n = 60;
    thread::spawn(move || loop {
        tx.send(GuiMessage::Hello("World!".to_string())).unwrap();

        let paint_tx_res = ptx.send(PaintMessage::RequestRepaint);
        match paint_tx_res {
            Ok(()) => println!("tx success"),
            Err(error) => println!("tx error: {error}"),
        }

        thread::sleep(Duration::from_secs(1));
        n -= 1;

        if n < 0 {
            break;
        }
    });

    // Application code
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let app = Application::new(rx);

    // WIP:
    let paint_rx_clone = Arc::clone(&final_paint_rx);

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            println!("Hello from inside CreationContext");
            let frame = cc.egui_ctx.clone();

            thread::spawn(move || {
                println!("Can threads print to stdout?");
                debug!("this is a debug {}", "message");

                loop {
                    println!("loop");
                    match paint_rx_clone.try_lock().unwrap().try_recv() {
                        Ok(msg) => match msg {
                            PaintMessage::RequestRepaint => {
                                println!("Repaint request received!");
                                frame.request_repaint();
                            }
                        },
                        Err(TryRecvError::Empty) => {
                            println!("Empty");
                        }
                        Err(TryRecvError::Disconnected) => {
                            println!("Channel disconnected");
                        }
                    }

                    thread::sleep(Duration::from_secs(1));
                }
            });

            Ok(Box::new(app))
        }),
    )
}
