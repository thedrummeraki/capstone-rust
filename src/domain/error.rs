use thiserror::Error;

pub type LoadBalancerResult<T> = Result<T, LoadBalancerError>;

#[derive(Error, Debug)]
pub enum LoadBalancerError {
    #[error("Invalid config error. Details: `{0}`.")]
    ConfigError(String),
    #[error("Generic Load balancer error occured: `{0}`.")]
    GenericError(Box<dyn std::error::Error>),
    #[error("Unknown error occured")]
    UnknowError,
}
