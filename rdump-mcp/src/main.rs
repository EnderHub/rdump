#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rdump_mcp::run_stdio().await
}
