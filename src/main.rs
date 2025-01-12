pub mod gui;
pub mod rpc;

use futures::{future, StreamExt};
use gui::Application;
use log::debug;
use rpc::{spawn, HelloServer, World};
use rpc::{GuiMessage, PaintMessage};
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
    thread,
    time::Duration,
};

use eframe::egui;
use tarpc::{
    server::{self, Channel},
    tokio_serde::formats::Json,
};
use tokio::sync::{
    mpsc::{channel, error::TryRecvError},
    Mutex,
};

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

        let listener = tarpc::serde_transport::tcp::listen(&addr, Json::default)
            .await
            .expect("whoops!");
        listener
            // Ignore accept errors.
            .filter_map(|r| future::ready(r.ok()))
            .map(server::BaseChannel::with_defaults)
            .map(|channel| {
                let server = HelloServer {
                    paint_tx: Arc::clone(&paint_tx_ref),
                    gui_tx: Arc::clone(&gui_tx_ref),
                };
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
