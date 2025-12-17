use flare::datanode;

#[tokio::main]
async fn main() {
    datanode::start().await;
}
