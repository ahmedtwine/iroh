//! Iroh Mesh - P2P Service Mesh for Kubernetes
//!
//! A service mesh implementation that uses iroh for P2P connectivity between
//! Kubernetes clusters, enabling direct encrypted connections with NAT traversal.

pub mod proxy;
pub mod agent;
pub mod discovery;
pub mod config;
pub mod error;

pub use error::{Error, Result};

use std::net::SocketAddr;
use iroh_base::NodeId;
use serde::{Deserialize, Serialize};

/// Represents a cluster in the mesh
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClusterId(pub String);

impl std::fmt::Display for ClusterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Information about a cluster in the mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub id: ClusterId,
    pub node_id: NodeId,
    pub relay_url: Option<String>,
    pub direct_addresses: Vec<SocketAddr>,
    pub services: Vec<ServiceInfo>,
}

/// Information about a service exposed by a cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub namespace: String,
    pub port: u16,
    pub protocol: String,
}

/// Configuration for cross-cluster service routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossClusterRoute {
    pub target_cluster: ClusterId,
    pub target_service: String,
    pub target_namespace: String,
    pub target_port: u16,
}

/// ALPN protocol identifier for mesh communication
pub const MESH_ALPN: &[u8] = b"iroh-mesh/v1";

/// Default port for the mesh proxy
pub const DEFAULT_PROXY_PORT: u16 = 15001;

/// Default port for the mesh agent
pub const DEFAULT_AGENT_PORT: u16 = 15002;