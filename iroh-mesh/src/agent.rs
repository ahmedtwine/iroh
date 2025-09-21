//! Mesh agent for cluster coordination and discovery

use crate::{
    config::AgentConfig, 
    discovery::DiscoveryManager, 
    ClusterInfo, 
    Error, 
    Result, 
    ServiceInfo
};
use iroh::{Endpoint, NodeAddr};
use iroh_base::SecretKey;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Agent that manages cluster discovery and coordination
#[derive(Debug)]
pub struct MeshAgent {
    config: AgentConfig,
    endpoint: Endpoint,
    discovery: Arc<DiscoveryManager>,
}

/// API response for cluster information
#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterStatusResponse {
    pub cluster_id: String,
    pub node_id: String,
    pub node_addr: String,
    pub services: Vec<ServiceInfo>,
    pub peer_clusters: Vec<String>,
}

impl MeshAgent {
    /// Create a new mesh agent
    pub async fn new(config: AgentConfig) -> Result<Self> {
        info!("Starting mesh agent for cluster: {}", config.cluster_id);

        // Load or generate secret key
        let secret_key = match &config.secret_key_path {
            Some(path) => {
                match std::fs::read_to_string(path) {
                    Ok(content) => content.trim().parse()
                        .map_err(|e| Error::Config(format!("Invalid secret key: {}", e)))?,
                    Err(_) => {
                        let key = SecretKey::generate(OsRng);
                        std::fs::write(path, format!("{:?}", key))?;
                        key
                    }
                }
            }
            None => SecretKey::generate(OsRng),
        };

        // Create iroh endpoint for agent communication
        let endpoint = Endpoint::builder()
            .secret_key(secret_key)
            .bind()
            .await
            .map_err(|e| Error::Network(format!("Failed to bind agent endpoint: {}", e)))?;

        info!("Agent endpoint created with NodeId: {}", endpoint.node_id());

        // Create discovery manager
        let discovery = Arc::new(DiscoveryManager::new(config.cluster_id.clone()).await?);

        Ok(Self {
            config,
            endpoint,
            discovery,
        })
    }

    /// Start the agent
    pub async fn run(&self) -> Result<()> {
        info!("Starting agent server on {}", self.config.bind_address);

        // Start periodic service discovery
        let discovery_task = self.start_service_discovery();

        // Start the HTTP API server
        let api_task = self.start_api_server();

        // Start cluster registration
        let registration_task = self.start_cluster_registration();

        // Run all tasks concurrently
        tokio::select! {
            result = discovery_task => {
                error!("Service discovery task ended: {:?}", result);
                result
            }
            result = api_task => {
                error!("API server task ended: {:?}", result);
                result
            }
            result = registration_task => {
                error!("Cluster registration task ended: {:?}", result);
                result
            }
        }
    }

    /// Start periodic service discovery
    async fn start_service_discovery(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(30));
        let discovery = self.discovery.clone();
        let namespace = self.config.kubernetes.namespace.clone();

        loop {
            interval.tick().await;
            
            match discovery.discover_local_services(namespace.as_deref()).await {
                Ok(services) => {
                    debug!("Discovered {} services", services.len());
                    // TODO: Update cluster registration with new services
                }
                Err(e) => {
                    warn!("Failed to discover services: {}", e);
                }
            }
        }
    }

    /// Start the HTTP API server
    async fn start_api_server(&self) -> Result<()> {
        // Simplified HTTP server for now - just bind to the address
        // In a real implementation, we'd use a proper HTTP framework
        info!("Agent API server would listen on {}", self.config.bind_address);
        
        // For now, just sleep to simulate the server running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    /// Start cluster registration and heartbeat
    async fn start_cluster_registration(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(60));
        let discovery = self.discovery.clone();
        let cluster_id = self.config.cluster_id.clone();
        let node_id = self.endpoint.node_id();
        let _node_addr = self.endpoint.node_addr();

        loop {
            interval.tick().await;

            // Get current services
            let services = match discovery.discover_local_services(
                self.config.kubernetes.namespace.as_deref()
            ).await {
                Ok(services) => services,
                Err(e) => {
                    warn!("Failed to get services for registration: {}", e);
                    continue;
                }
            };

            // Create cluster info
            let cluster_info = ClusterInfo {
                id: cluster_id.clone(),
                node_id,
                relay_url: None, // TODO: Get from endpoint
                direct_addresses: Vec::new(), // TODO: Get from endpoint
                services,
            };

            // Register cluster
            if let Err(e) = discovery.register_cluster(cluster_info).await {
                warn!("Failed to register cluster: {}", e);
            } else {
                debug!("Cluster registration updated");
            }
        }
    }

    /// Get cluster status for API
    pub async fn get_status(&self) -> ClusterStatusResponse {
        let services = self.discovery.discover_local_services(None).await.unwrap_or_default();
        let clusters = self.discovery.list_clusters().await;
        let peer_clusters: Vec<String> = clusters.iter()
            .filter(|c| c.id != self.config.cluster_id)
            .map(|c| c.id.to_string())
            .collect();

        ClusterStatusResponse {
            cluster_id: self.config.cluster_id.to_string(),
            node_id: self.endpoint.node_id().to_string(),
            node_addr: "pending".to_string(), // Will be updated when node_addr is available
            services,
            peer_clusters,
        }
    }

    /// Get the NodeAddr for this agent
    pub async fn node_addr(&self) -> Option<NodeAddr> {
        use n0_watcher::Watcher;
        self.endpoint.node_addr().get()
    }
}