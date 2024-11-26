pub type LoadBalancerResult<T> = Result<T, LoadBalancerError>;

#[derive(Debug)]
pub enum LoadBalancerError {
    ConfigError(String),
    GenericError(Box<dyn std::error::Error>),
    UnknowError,
}
