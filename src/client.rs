use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tarpc::{client, context, tokio_serde::formats::Json};

#[tarpc::service]
trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("client main start");

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
    let transport = tarpc::serde_transport::tcp::connect(&addr, Json::default);
    let client = WorldClient::new(client::Config::default(), transport.await?).spawn();

    println!("About to request...");

    client
        .hello(context::current(), "Hello from the client".to_string())
        .await?;

    println!("client main done");

    Ok(())
}
