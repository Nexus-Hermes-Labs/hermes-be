pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Auth Service is starting...");
    Ok(())
}
