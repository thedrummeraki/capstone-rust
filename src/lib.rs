use domain::error::LoadBalancerResult;

pub mod domain;

mod cli;

pub async fn run() -> LoadBalancerResult<()> {
    cli::run().await
}
