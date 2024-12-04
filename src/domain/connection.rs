pub type WorkerClient = String;

#[derive(Debug, Clone)]
pub struct ActiveConnection {
    // The client that is connected to the load balancer.
    pub client: WorkerClient,
}

impl PartialEq for ActiveConnection {
    fn eq(&self, other: &Self) -> bool {
        self.client.eq(&other.client)
    }
}
