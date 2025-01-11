use tarpc::{client, context, tokio_serde::formats::Json};

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
    println!("client main start");

    let transport = tarpc::serde_transport::tcp::connect("[::1]:5000", Json::default);
    let client = WorldClient::new(client::Config::default(), transport.await?).spawn();

    client
        .hello(context::current(), "Hello from the client".to_string())
        .await?;

    println!("client main done");

    Ok(())
}
