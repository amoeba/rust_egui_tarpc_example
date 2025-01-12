use futures::{future, StreamExt};
use log::debug;
use std::{
    future::Future, net::{IpAddr, Ipv4Addr}, sync::{
        Arc,
    }, thread, time::Duration
};

use tarpc::{context, server::{self, Channel}, tokio_serde::formats::Json};
use tokio::sync::{mpsc::{channel, error::TryRecvError, Receiver, Sender}, Mutex};
use eframe::egui;

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
  }

pub enum GuiMessage {
    Hello(String),
    UpdateString(String),
}

pub enum PaintMessage {
    RequestRepaint,
}

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
                    },
                    GuiMessage::UpdateString(value) => {
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

// tarpc
#[tarpc::service]
trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
    async fn update_string(value: String) -> String;
}

#[derive(Clone)]
pub struct HelloServer {
    paint_tx: Arc<Mutex<Sender<PaintMessage>>>,
    gui_tx: Arc<Mutex<Sender<GuiMessage>>>,
}

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {

        match self.paint_tx.lock().await.send(PaintMessage::RequestRepaint).await {
                Ok(()) => println!("Repaint Requested"),
                Err(error) => println!("tx error: {error}"),
        }

        format!("Hello, {name}!")
    }

    async fn update_string(self, context: ::tarpc::context::Context,value:String) -> String {
        println!("in tarpc server update_string handler!");

        value
    }
}

fn main() -> eframe::Result {
    env_logger::init();

    //////////////
    // CHANNELS //
    //////////////

    // Channel for sending updates related to mutating the GUI state
    let (gui_tx, gui_rx) = channel::<GuiMessage>(32);
    let gui_rx_ref = Arc::new(Mutex::new(gui_rx));
    let gui_tx_ref = Arc::new(Mutex::new(gui_tx));

    // Maybe removable channel for sending a request to the GUI to paint
    let (paint_tx, paint_rx) = channel::<PaintMessage>(32);
    let paint_rx_ref = Arc::new(Mutex::new(paint_rx));
    let paint_tx_ref = Arc::new(Mutex::new(paint_tx));

    ///////////
    // TARPC //
    ///////////

    let runtime = tokio::runtime::Runtime::new().unwrap();


    runtime.spawn(async move {
        let addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

        let listener = tarpc::serde_transport::tcp::listen(&addr, Json::default).await.expect("whoops!");
        listener
            // Ignore accept errors.
            .filter_map(|r| future::ready(r.ok()))
            .map(server::BaseChannel::with_defaults)
            .map(|channel| {
                println!("got a client request!");
                let server = HelloServer {paint_tx: Arc::clone(&paint_tx_ref), gui_tx: Arc::clone(&gui_tx_ref) };
                channel.execute(server.serve()).for_each(spawn)
            })
            .buffer_unordered(10)
            .for_each(|_| async {})
            .await;
    });

    // Application code
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let app = Application::new(Arc::clone(&gui_rx_ref));


    // TODO: Rename
    let x = Arc::clone(&paint_rx_ref);
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            println!("Hello from inside CreationContext");
            let frame = cc.egui_ctx.clone();

            thread::spawn(move || {
                debug!("Spawning app repaint poll thread");

                loop {
                    match x.try_lock().unwrap().try_recv() {
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
