//! Binary for the iroh-mesh agent

use clap::Parser;
use iroh_mesh::{
    config::{AgentConfig, load_config},
    agent::MeshAgent,
    ClusterId,
    DEFAULT_AGENT_PORT,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(
    name = "iroh-agent",
    about = "Iroh mesh agent for cluster coordination",
    version
)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Bind address for the agent API
    #[arg(short, long, default_value_t = ([127, 0, 0, 1], DEFAULT_AGENT_PORT).into())]
    bind: SocketAddr,

    /// Cluster ID
    #[arg(long, default_value = "default")]
    cluster_id: String,

    /// Path to secret key file
    #[arg(long)]
    secret_key: Option<PathBuf>,

    /// Kubernetes namespace to watch
    #[arg(long)]
    namespace: Option<String>,

    /// Enable DNS discovery
    #[arg(long)]
    enable_dns: bool,

    /// Enable mDNS discovery
    #[arg(long)]
    enable_mdns: bool,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::new(&args.log_level))
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting iroh-mesh agent");

    // Load configuration
    let config = if let Some(config_path) = args.config {
        load_config::<AgentConfig>(&config_path)?
    } else {
        // Create config from CLI args
        let mut config = AgentConfig::default();
        config.bind_address = args.bind;
        config.cluster_id = ClusterId(args.cluster_id);
        config.secret_key_path = args.secret_key;
        config.kubernetes.namespace = args.namespace;
        config.discovery.enable_dns = args.enable_dns;
        config.discovery.enable_mdns = args.enable_mdns;
        config
    };

    info!("Agent configuration: {:?}", config);

    // Create and run agent
    let agent = MeshAgent::new(config).await?;
    
    info!("Agent NodeAddr: {:?}", agent.node_addr().await);

    if let Err(e) = agent.run().await {
        error!("Agent error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}