//! Configuration for iroh-mesh components

use crate::{ClusterId, Error, Result};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

/// Configuration for the mesh proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Address to bind the proxy server to
    pub bind_address: SocketAddr,

    /// Cluster ID for this proxy
    pub cluster_id: ClusterId,

    /// Path to the iroh secret key file
    pub secret_key_path: Option<PathBuf>,

    /// Enable traffic interception
    pub enable_interception: bool,

    /// Kubernetes configuration
    pub kubernetes: KubernetesConfig,
}

/// Configuration for the mesh agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Address to bind the agent API to
    pub bind_address: SocketAddr,

    /// Cluster ID for this agent
    pub cluster_id: ClusterId,

    /// Path to the iroh secret key file
    pub secret_key_path: Option<PathBuf>,

    /// Kubernetes configuration
    pub kubernetes: KubernetesConfig,

    /// Discovery configuration
    pub discovery: DiscoveryConfig,
}

/// Kubernetes client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Namespace to watch for services (None = all namespaces)
    pub namespace: Option<String>,

    /// Service account token path
    pub token_path: Option<PathBuf>,

    /// CA certificate path
    pub ca_cert_path: Option<PathBuf>,
}

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable DNS discovery
    pub enable_dns: bool,

    /// Enable mDNS discovery
    pub enable_mdns: bool,

    /// Custom discovery endpoints
    pub endpoints: Vec<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            bind_address: ([127, 0, 0, 1], crate::DEFAULT_PROXY_PORT).into(),
            cluster_id: ClusterId("default".to_string()),
            secret_key_path: None,
            enable_interception: false,
            kubernetes: KubernetesConfig::default(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            bind_address: ([127, 0, 0, 1], crate::DEFAULT_AGENT_PORT).into(),
            cluster_id: ClusterId("default".to_string()),
            secret_key_path: None,
            kubernetes: KubernetesConfig::default(),
            discovery: DiscoveryConfig::default(),
        }
    }
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        Self {
            namespace: None,
            token_path: Some("/var/run/secrets/kubernetes.io/serviceaccount/token".into()),
            ca_cert_path: Some("/var/run/secrets/kubernetes.io/serviceaccount/ca.crt".into()),
        }
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_dns: true,
            enable_mdns: false,
            endpoints: Vec::new(),
        }
    }
}

/// Load configuration from file
pub fn load_config<T: for<'de> Deserialize<'de>>(path: &PathBuf) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    let config = serde_yaml::from_str(&content)
        .map_err(|e| Error::Config(format!("Failed to parse config file: {}", e)))?;
    Ok(config)
}

/// Save configuration to file
pub fn save_config<T: Serialize>(config: &T, path: &PathBuf) -> Result<()> {
    let content = serde_yaml::to_string(config)
        .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;
    std::fs::write(path, content)?;
    Ok(())
}
