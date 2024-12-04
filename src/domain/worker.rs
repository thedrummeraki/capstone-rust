use std::{collections::HashSet, fmt::Display, str::FromStr};

use hyper::Uri;

use super::{
    connection::{ActiveConnection, WorkerClient},
    error::{LoadBalancerError, LoadBalancerResult},
};

const MIN_RECOMMENDED_WORKERS_COUNT: i8 = 2;

#[derive(Debug, Clone, Default, PartialEq)]
struct ActiveConnections {
    data: Vec<ActiveConnection>,
}

impl ActiveConnections {
    pub async fn try_add_connection(
        &mut self,
        connection: ActiveConnection,
    ) -> LoadBalancerResult<()> {
        match self.data.contains(&connection) {
            true => Err(LoadBalancerError::AlreadyConnected(connection.client)),
            false => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Worker {
    address: Uri,
    status: WorkerStatus,
    connections: ActiveConnections,
}

impl TryFrom<String> for Worker {
    type Error = LoadBalancerError;

    fn try_from(address: String) -> Result<Self, Self::Error> {
        Ok(Self {
            address: Uri::from_str(&address).map_err(|_| {
                LoadBalancerError::GenericError(format!("Invalid URI {address}").into())
            })?,
            status: WorkerStatus::default(),
            connections: ActiveConnections::default(),
        })
    }
}

impl Worker {
    pub async fn ack_connection(&mut self, client: WorkerClient) -> LoadBalancerResult<()> {
        let connection = ActiveConnection { client };
        self.connections.try_add_connection(connection).await?;

        Ok(())
    }

    pub fn uri(&self) -> Uri {
        self.address.clone()
    }

    pub fn accepts_connection(&self) -> bool {
        matches!(self.status, WorkerStatus::Up)
    }
}

impl Display for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<# Worker ({} -- status:{:?} connections:{})>",
            self.address,
            self.status,
            self.connections.data.len()
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum WorkerStatus {
    #[default]
    Pending,
    Starting,
    Up,
    Down,
    Failed,
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WorkersList(Vec<Worker>);

impl WorkersList {
    pub fn parse(list: Vec<String>) -> LoadBalancerResult<Self> {
        let set: HashSet<String> = list.into_iter().collect();
        let list: Vec<String> = set.into_iter().collect();

        if list.is_empty() {
            return Err(LoadBalancerError::ConfigError(
                "At least 1 worker must be specified.".into(),
            ));
        }

        if list.len() < MIN_RECOMMENDED_WORKERS_COUNT.try_into().unwrap() {
            eprintln!("[WARN] We recommend you to specify at least {MIN_RECOMMENDED_WORKERS_COUNT} different workers.")
        }

        let workers = list
            .iter()
            .filter_map(|address| match Worker::try_from(address.to_owned()) {
                Err(e) => {
                    eprintln!("[WARN] Could not add worker to workers list: {e}");
                    None
                }
                Ok(worker) => Some(worker),
            })
            .collect();

        Ok(Self(workers))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Worker> {
        self.0.get(index)
    }
}

impl Display for WorkersList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().enumerate().for_each(|(i, worker)| {
            writeln!(f, "--> [{}] {worker}", i + 1)
                .expect("could not write WorkersList to formatter");
        });

        Ok(())
    }
}
