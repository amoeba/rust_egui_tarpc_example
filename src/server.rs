// Copyright 2018 Google LLC
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use futures::{future, prelude::*};
use service::World;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tarpc::{
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};

#[derive(Clone)]
struct HelloServer(SocketAddr);

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}! You are connected from {}", self.0)
    }

    async fn handle_recvfrom(self, _: context::Context, data: Vec<u8>) -> String {
        println!("handle_recvfrom: {data:?}");

        for (_, byte) in data.iter().enumerate() {
            println!("got {byte:02X}");
        }

        "got it".to_string()
    }
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), 5000);

    let listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;

    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            let server = HelloServer(channel.transport().peer_addr().unwrap());
            channel.execute(server.serve()).for_each(spawn)
        })
        .buffer_unordered(1)
        .for_each(|_| async {})
        .await;

    Ok(())
}
