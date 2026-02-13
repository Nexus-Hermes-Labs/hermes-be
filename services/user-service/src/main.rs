
pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod state;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    // Run the application
    if let Err(e) = bootstrap::run().await {
        eprintln!("❌ Application error: {}", e);
        std::process::exit(1);
    }
}
