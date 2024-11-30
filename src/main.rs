use captsone_rust::{cli, domain::error::LoadBalancerResult};

#[tokio::main]
async fn main() -> LoadBalancerResult<()> {
    cli::run().await
}
