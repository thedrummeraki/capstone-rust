use std::io;

use thiserror::Error;

use super::connection::WorkerClient;

pub type LoadBalancerResult<T> = Result<T, LoadBalancerError>;

#[derive(Error, Debug)]
pub enum LoadBalancerError {
    #[error("Invalid config error. Details: `{0}`.")]
    ConfigError(String),
    #[error("Generic Load balancer error occured: `{0}`.")]
    GenericError(Box<dyn std::error::Error>),
    #[error("TCP error: {0}")]
    TcpListenerError(io::Error),
    #[error("Hyper error {0}")]
    HyperError(hyper::Error),
    #[error("HTTP error {0}")]
    HttpError(hyper::http::Error),
    #[error("There already is an active connection from `{0}")]
    AlreadyConnected(WorkerClient),
    #[error("Invalid URI specified `{0}")]
    InvalidUri(hyper::http::uri::InvalidUri),
    #[error("Unknown error occured")]
    UnknowError,
}
