use std::{
    net::{IpAddr, Ipv6Addr},
    sync::mpsc as std_mpsc,
    thread,
};

use futures::StreamExt;
use tarpc::{
    server::{self, Channel},
    tokio_serde::formats::Json,
};
use tokio::runtime::Runtime;

use service::{spawn, Application, HelloServer, World};

fn main() -> eframe::Result {
    env_logger::init();

    let (gui_tx, gui_rx) = std_mpsc::channel();

    let _hdl = thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        rt.block_on(async move {
            let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), 5000);

            let listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default)
                .await
                .expect("Failed to start TCP listener");

            listener
                .filter_map(|r| async { r.ok() })
                .map(server::BaseChannel::with_defaults)
                .map(|channel| {
                    let hello_server = HelloServer {
                        gui_tx: gui_tx.clone(),
                    };
                    channel.execute(hello_server.serve()).for_each(spawn)
                })
                .buffer_unordered(10)
                .for_each(|_| async {})
                .await;
        });
    });

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::new(Application::new(gui_rx)))),
    )
}
