use std::fmt::Display;

use super::error::{LoadBalancerError, LoadBalancerResult};

#[derive(Debug, PartialEq, Clone)]
pub struct WorkersList(Vec<String>);

impl WorkersList {
    pub fn parse(list: Vec<String>) -> LoadBalancerResult<Self> {
        if list.is_empty() {
            return Err(LoadBalancerError::ConfigError(
                "At least 1 worker must be specified.".into(),
            ));
        }

        Ok(Self(list))
    }
}

impl AsRef<Vec<String>> for WorkersList {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}

impl Display for WorkersList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let workers = self.as_ref().to_vec();
        workers.iter().enumerate().for_each(|(i, worker)| {
            writeln!(f, "--> [{}] {worker}", i + 1)
                .expect("could not write WorkersList to formatter");
        });

        Ok(())
    }
}
