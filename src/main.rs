use captsone_rust::{cli, domain::error::LoadBalancerResult};

#[tokio::main]
async fn main() -> LoadBalancerResult<()> {
    if let Err(e) = cli::run().await {
        eprintln!("{e}");
    }
    Ok(())
}
