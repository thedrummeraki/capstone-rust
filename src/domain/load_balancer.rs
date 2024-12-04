use std::{net::SocketAddr, str::FromStr, sync::Arc};

use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{
    body::{Bytes, Incoming},
    http::response::Builder as ResponseBuilder,
    server::conn::http1,
    service::service_fn,
    Request, Response, StatusCode, Uri,
};
use hyper_util::rt::TokioIo;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use super::{
    config::Config,
    error::{LoadBalancerError, LoadBalancerResult},
    worker::{Worker, WorkersList},
};

#[derive(Debug, Clone)]
struct LoadBalancer {
    workers_list: WorkersList,
    next_worker_selector: Arc<RwLock<NextWorkerSelector>>,
}

impl LoadBalancer {
    pub fn new(workers_list: WorkersList) -> LoadBalancerResult<Self> {
        Ok(LoadBalancer {
            workers_list,
            next_worker_selector: Arc::new(RwLock::new(NextWorkerSelector::new())),
        })
    }

    pub async fn forward_request(
        &self,
        req: Request<Incoming>,
    ) -> LoadBalancerResult<Response<BoxBody<Bytes, hyper::Error>>> {
        match self.get_worker().await {
            None => self.handle_no_available_workers().await,
            Some(worker) => self.handle_forward_worker(req, worker).await,
        }
    }

    pub async fn get_worker(&self) -> Option<Worker> {
        let mut selector = self.next_worker_selector.write().await;

        selector.select_next_worker(self.workers_list.clone()).await
    }

    async fn handle_no_available_workers(
        &self,
    ) -> LoadBalancerResult<Response<BoxBody<Bytes, hyper::Error>>> {
        let response = self
            .build_response()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(
                Full::new(Bytes::from("value"))
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap();

        Ok(response)
    }

    async fn handle_forward_worker(
        &self,
        req: Request<Incoming>,
        worker: Worker,
    ) -> LoadBalancerResult<Response<BoxBody<Bytes, hyper::Error>>> {
        let uri = worker.uri();
        let host = uri.host().expect("worker uri has no host");
        let port = uri.port_u16().unwrap_or(80);
        let method = req.method().clone();
        let headers = req.headers().clone();
        let scheme = uri
            .scheme()
            .cloned()
            .unwrap_or("http".parse::<hyper::http::uri::Scheme>().unwrap());

        let mut address = format!("{scheme}://{host}:{port}");
        if let Some(path_and_query) = req.uri().path_and_query() {
            address.push_str(path_and_query.as_str());
        }

        println!("[{method}] Forwarding to '{address}'");

        let body = req
            .collect()
            .await
            .map_err(LoadBalancerError::HyperError)?
            .aggregate();

        let new_uri: Uri = Uri::from_str(&address).map_err(LoadBalancerError::InvalidUri)?;

        let stream = TcpStream::connect((host, port))
            .await
            .map_err(LoadBalancerError::TcpListenerError)?;
        let io = TokioIo::new(stream);

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
            .await
            .map_err(LoadBalancerError::HyperError)?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let authority = new_uri.authority().unwrap().clone();
        let mut req = Request::builder()
            .method(method)
            .uri(new_uri)
            .header(hyper::header::HOST, authority.as_str())
            .body(Full::new(body))
            .map_err(LoadBalancerError::HttpError)?;

        // Add headers from the incoming REQUEST
        for (key, value) in headers.iter() {
            req.headers_mut().insert(key, value.clone());
        }

        sender
            .send_request(req)
            .await
            .map(|incoming_res| {
                // Add the headers from the incoming RESPONSE
                let mut res = Response::builder();
                if let Some(res_headers) = res.headers_mut() {
                    for (key, value) in incoming_res.headers().iter() {
                        res_headers.insert(key, value.clone());
                    }
                }
                res.body(incoming_res.boxed()).expect("could not build res")
            })
            .map_err(LoadBalancerError::HyperError)
    }

    fn build_response(&self) -> ResponseBuilder {
        Response::builder().header("X-Proxied-From", "rusty-load-balancer")
    }

    pub async fn update_selection_strategy(&self, selection_strategy: WorkerSelectionStrategy) {
        println!("Updating load balancer worker selection stategy to: {selection_strategy:?}");
        let mut selector = self.next_worker_selector.write().await;
        selector.selection_strategy = selection_strategy;
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Index {
    n: usize,
}

impl Index {
    pub fn set(&mut self, n: usize) {
        self.n = n;
    }
}

#[derive(Debug)]
struct NextWorkerSelector {
    index: Index,
    selection_strategy: WorkerSelectionStrategy,
}

#[derive(Debug, Default, Clone, Copy)]
enum WorkerSelectionStrategy {
    #[default]
    RoundRobin,
}

impl NextWorkerSelector {
    pub fn new() -> Self {
        Self {
            index: Index::default(),
            selection_strategy: WorkerSelectionStrategy::default(),
        }
    }

    pub async fn select_next_worker(&mut self, workers_list: WorkersList) -> Option<Worker> {
        match self.selection_strategy {
            WorkerSelectionStrategy::RoundRobin => {
                Some(self.select_round_robin(workers_list).await)
            }
        }
    }

    async fn select_round_robin(&mut self, workers_list: WorkersList) -> Worker {
        let len = workers_list.len();

        let next_index = (self.index.n + 1) % len;
        self.index.set(next_index);

        let worker = workers_list
            .get(next_index)
            .expect("Unexpected non-existant worker from round-robin selection strategy");

        worker.clone()
    }
}

async fn handle(
    req: Request<Incoming>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match load_balancer.read().await.forward_request(req).await {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!("Forward request error occured: {e:?}");
            Ok(Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(
                    Full::new(Bytes::from(format!("Unexpected error: {e}")))
                        .map_err(|never| match never {})
                        .boxed(),
                )
                .unwrap())
        }
    }
}

pub async fn run(config: Config) -> LoadBalancerResult<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr)
        .await
        .map_err(LoadBalancerError::TcpListenerError)?;

    let instance: LoadBalancer = LoadBalancer::new(config.worker_hosts.clone())?;
    instance
        .update_selection_strategy(WorkerSelectionStrategy::RoundRobin)
        .await;

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .map_err(LoadBalancerError::TcpListenerError)?;

        let load_balancer = Arc::new(RwLock::new(instance.clone()));

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            [
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req| handle(req, load_balancer.clone())),
                    )
                    .await
                {
                    eprintln!("Error {err}")
                },
            ]
        });
    }
}
