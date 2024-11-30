use crate::domain::{config::Config, error::LoadBalancerResult, load_balancer};

pub async fn exec(config: Config) -> LoadBalancerResult<()> {
    load_balancer::run(config).await
}
