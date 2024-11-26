use captsone_rust::{domain::error::LoadBalancerResult, run};

#[tokio::main]
async fn main() -> LoadBalancerResult<()> {
    run().await
}
