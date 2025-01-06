use service::WorldClient;
use tarpc::{client, context, tokio_serde::formats::Json};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = tarpc::serde_transport::tcp::connect("[::1]:5000", Json::default);

    let client = WorldClient::new(client::Config::default(), transport.await?).spawn();

    let _hello = client.hello(context::current(), "Stim".to_string()).await?;

    let data = vec![0x60, 0x61, 0x62];
    let _hello = client.handle_recvfrom(context::current(), data).await?;

    Ok(())
}
