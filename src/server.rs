use std::net::{IpAddr, Ipv4Addr};

use futures::{future, StreamExt};
use tarpc::{context, server::{self, Channel}, tokio_serde::formats::Json};

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
  let listener = tarpc::serde_transport::tcp::listen(&addr, Json::default)
      .await
      .expect("Failed to start TCP listener");

  println!("Server listening on {}", addr.0);

  listener
      .filter_map(|r| future::ready(r.ok()))
      .for_each(|transport| {
          let server = HelloServer;
          async move {
              println!("Hello from inside tarpc foreach");
              let stream_fut = server::BaseChannel::with_defaults(transport)
                  .execute(server.serve());
              tokio::spawn(stream_fut.for_each(|x| {
                println!("hi");
                future::ready(())}));
          }
      })
      .await;

  Ok(())
}
