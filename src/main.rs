#[tokio::main]
async fn main() {
    let my_ip = dyndns::get_public_ip_address().await.unwrap();
    print!("{:?}\n", &my_ip);
}
