mod serve;

use clap::{Args, Parser, Subcommand};

use crate::domain::{
    config::Config,
    error::{LoadBalancerError, LoadBalancerResult},
    worker::WorkersList,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    // #[arg(short, long, action = clap::ArgAction::Count)]
    // debug: u8
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts the load balancer
    Serve(Serve),
}

#[derive(Args)]
struct Serve {
    /// Specify the port to run the server on.
    #[clap(short, long, default_value = "1337")]
    port: u16,

    #[clap(short, long)]
    worker: Vec<String>,
}

impl TryFrom<Serve> for Config {
    type Error = LoadBalancerError;

    fn try_from(val: Serve) -> LoadBalancerResult<Self> {
        Ok(Config {
            port: val.port,
            address: String::from("0.0.0.0"),
            worker_hosts: WorkersList::parse(val.worker)?,
        })
    }
}

pub async fn run() -> LoadBalancerResult<()> {
    let cli = Cli::parse();
    match cli.commands {
        Commands::Serve(serve) => serve::exec(serve.try_into()?).await,
    }
}
