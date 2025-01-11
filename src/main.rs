use futures::{future, StreamExt};
use log::debug;
use tarpc::{client, context, serde_transport::tcp, server::{self, Channel}, tokio_serde::formats::Json, tokio_util::codec::LengthDelimitedCodec, transport};
use tokio::{net::TcpListener, runtime::Runtime, spawn};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr}, sync::{
        mpsc::{channel, Receiver, TryRecvError},
        Arc, Mutex,
    }, thread, time::Duration
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

// tarpc

#[tarpc::service]
trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
}

#[derive(Clone)]
struct HelloServer;

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}!")
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

    // WIP. Replace this with RPC code eventually.
    let tx = tx.clone();
    let ptx = paint_tx.clone();
    let mut n = 60;
    thread::spawn(move || loop {
        tx.send(GuiMessage::Hello("World!".to_string())).unwrap();

        let paint_tx_res = ptx.send(PaintMessage::RequestRepaint);
        match paint_tx_res {
            Ok(()) => println!("Repaint Requested"),
            Err(error) => println!("tx error: {error}"),
        }

        thread::sleep(Duration::from_secs(1));
        n -= 1;

        if n < 0 {
            break;
        }
    });

    // tarpc code
    let runtime = tokio::runtime::Runtime::new().unwrap();

    runtime.spawn(async {
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
        let listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default)
            .await
            .expect("Failed to start TCP listener");

        println!("Server listening on {}", server_addr);

        listener
            .filter_map(|r| future::ready(r.ok()))
            .for_each(|transport| {
                let server = HelloServer;
                async move {
                    println!("Hello from inside tarpc foreach");
                    let st = server::BaseChannel::with_defaults(transport)
                        .execute(server.serve());
                    tokio::spawn(st.for_each(|_| future::ready(())));
                }
            })
            .await;
    });

    // Application code
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let app = Application::new(rx);

    // Note: Is this really necessary? I did this so I could keep the Receiver
    // alive inside my Application's polling loop
    let final_paint_rx = Arc::new(Mutex::new(paint_rx));
    let paint_rx_clone = Arc::clone(&final_paint_rx);

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            println!("Hello from inside CreationContext");
            let frame = cc.egui_ctx.clone();

            thread::spawn(move || {
                debug!("Spawning app repaint poll thread");

                loop {
                    match paint_rx_clone.try_lock().unwrap().try_recv() {
                        Ok(msg) => match msg {
                            PaintMessage::RequestRepaint => {
                                println!("Repaint request received!");
                                frame.request_repaint();
                            }
                        },
                        Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => {
                            println!("Channel disconnected");
                            break;
                        }
                    }

                    // ? 60FPS
                    thread::sleep(Duration::from_millis(16));
                }
            });

            Ok(Box::new(app))
        }),
    )
}
