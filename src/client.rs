pub mod rpc;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::{self},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rpc::WorldClient;
use tarpc::{client, context, tokio_serde::formats::Json};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
    let transport = tarpc::serde_transport::tcp::connect(&addr, Json::default);
    let client = WorldClient::new(client::Config::default(), transport.await?).spawn();

    // Say hello
    let resp = client
        .hello(context::current(), "Hello from the client".to_string())
        .await?;
    println!("Hello response is {resp}");

    // Update a string value
    let resp = client
        .update_string(context::current(), "this was set by the client".to_string())
        .await?;
    println!("Update string response is {resp}");

    // Write logs
    let mut i = 1024;
    loop {
        if i < 0 {
            break;
        }

        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        let resp = client
            .append_log(context::current(), current_timestamp.to_string())
            .await?;
        println!("Append log response is {resp}");

        thread::sleep(Duration::from_secs(1));
        i = i - 1;
    }

    Ok(())
}
