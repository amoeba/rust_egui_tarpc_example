use std::{future::Future, net::{IpAddr, Ipv4Addr}};
use futures::{future, StreamExt};
use tarpc::{
    context,
    server::{self, Channel},
    tokio_serde::formats::Json,
};

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
async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
  tokio::spawn(fut);
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

    let listener = tarpc::serde_transport::tcp::listen(&addr, Json::default).await?;
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            let server = HelloServer;
            channel.execute(server.serve()).for_each(spawn)
        })
        .buffer_unordered(2)
        .for_each(|_| async {})
        .await;

  Ok(())
}
