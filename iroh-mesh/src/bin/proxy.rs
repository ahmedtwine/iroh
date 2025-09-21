//! Binary for the iroh-mesh proxy

use clap::Parser;
use iroh_mesh::{
    config::{ProxyConfig, load_config},
    proxy::MeshProxy,
    ClusterId,
    DEFAULT_PROXY_PORT,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(
    name = "iroh-proxy",
    about = "Iroh mesh proxy for cross-cluster communication",
    version
)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Bind address for the proxy
    #[arg(short, long, default_value_t = ([127, 0, 0, 1], DEFAULT_PROXY_PORT).into())]
    bind: SocketAddr,

    /// Cluster ID
    #[arg(long, default_value = "default")]
    cluster_id: String,

    /// Path to secret key file
    #[arg(long)]
    secret_key: Option<PathBuf>,

    /// Enable traffic interception
    #[arg(long)]
    enable_interception: bool,

    /// Kubernetes namespace to watch
    #[arg(long)]
    namespace: Option<String>,

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

    info!("Starting iroh-mesh proxy");

    // Load configuration
    let config = if let Some(config_path) = args.config {
        load_config::<ProxyConfig>(&config_path)?
    } else {
        // Create config from CLI args
        let mut config = ProxyConfig::default();
        config.bind_address = args.bind;
        config.cluster_id = ClusterId(args.cluster_id);
        config.secret_key_path = args.secret_key;
        config.enable_interception = args.enable_interception;
        config.kubernetes.namespace = args.namespace;
        config
    };

    info!("Proxy configuration: {:?}", config);

    // Create and run proxy
    let proxy = MeshProxy::new(config).await?;
    
    info!("Proxy NodeId: {}", proxy.node_id());
    info!("Proxy NodeAddr: {:?}", proxy.node_addr().await);

    if let Err(e) = proxy.run().await {
        error!("Proxy error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}