use std::{convert::Infallible, net::SocketAddr, str::FromStr, sync::Arc};

use hyper::{
    client::ResponseFuture,
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server, Uri,
};
use tokio::sync::RwLock;

use super::{config::Config, error::LoadBalancerResult};

struct LoadBalancer {
    client: Client<hyper::client::HttpConnector>,
    worker_hosts: Vec<String>,
    current_worker: usize,
}

impl LoadBalancer {
    pub fn new(worker_hosts: Vec<String>) -> Result<Self, String> {
        if worker_hosts.is_empty() {
            return Err("No worker hosts provided".into());
        }

        Ok(LoadBalancer {
            client: Client::new(),
            worker_hosts,
            current_worker: 0,
        })
    }

    pub fn forward_request(&mut self, req: Request<Body>) -> ResponseFuture {
        let mut worker_uri = self.get_worker().to_owned();

        // Extract the path and query from the original request
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        // Create a new URI from the worker URI
        let new_uri = Uri::from_str(&worker_uri).unwrap();

        // Extract the headers from the original request
        let headers = req.headers().clone();

        // Clone the original request's headers and method
        let mut new_req = Request::builder()
            .method(req.method())
            .uri(new_uri)
            .body(req.into_body())
            .expect("request builder");

        // Copy headers from the original request
        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        self.client.request(new_req)
    }

    fn get_worker(&mut self) -> &str {
        // Use a round-robin strategy to select a worker
        let worker = self.worker_hosts.get(self.current_worker).unwrap();
        self.current_worker = (self.current_worker + 1) % self.worker_hosts.len();
        worker
    }
}

async fn handle(
    req: Request<Body>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<Body>, hyper::Error> {
    load_balancer.write().await.forward_request(req).await
}

pub async fn run(config: Config) -> LoadBalancerResult<()> {
    let worker_hosts = config.worker_hosts.to_owned();
    println!("Serving loading balancer on:");
    println!("{worker_hosts}");

    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(worker_hosts.as_ref().to_vec()).expect("failed to create load balancer"),
    ));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], config.port));

    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let load_balancer = load_balancer.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, load_balancer.clone()))) }
    }));

    if let Err(e) = server.await {
        println!("error: {}", e);
    }

    Ok(())
}
