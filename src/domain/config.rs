use super::worker::WorkersList;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub port: u16,
    pub address: String,
    pub worker_hosts: WorkersList,
}
