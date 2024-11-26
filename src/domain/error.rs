pub type LoadBalancerResult<T> = Result<T, LoadBalancerError>;

#[derive(Debug)]
pub enum LoadBalancerError {
    GenericError(Box<dyn std::error::Error>),
    UnknowError,
}
