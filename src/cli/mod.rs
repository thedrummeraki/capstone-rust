use crate::domain::{error::LoadBalancerResult, load_balancer};

pub async fn run() -> LoadBalancerResult<()> {
    load_balancer::run().await
}
