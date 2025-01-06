#[tarpc::service]
pub trait World {
    async fn hello(name: String) -> String;
    async fn handle_recvfrom(data: Vec<u8>) -> String;
}
