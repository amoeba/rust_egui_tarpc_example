use service::WorldClient;
use tarpc::{client, context, tokio_serde::formats::Json};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("client main");

    let transport = tarpc::serde_transport::tcp::connect("[::1]:5000", Json::default);
    let client = WorldClient::new(client::Config::default(), transport.await?).spawn();

    client.hello(context::current(), "Stim".to_string()).await?;

    Ok(())
}
