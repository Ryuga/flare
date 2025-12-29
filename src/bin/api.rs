use flare::api;

#[tokio::main]
async fn main() {
    api::start().await;
}
