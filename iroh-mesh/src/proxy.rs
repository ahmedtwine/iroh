//! Service mesh proxy implementation using iroh

use crate::{config::ProxyConfig, discovery::DiscoveryManager, Error, Result, MESH_ALPN};
use iroh::{protocol::ProtocolHandler, Endpoint, NodeAddr};
use iroh_base::{NodeId, SecretKey};
use rand::rngs::OsRng;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

/// Main proxy server that handles cross-cluster traffic
#[derive(Debug)]
pub struct MeshProxy {
    config: ProxyConfig,
    endpoint: Endpoint,
    discovery: Arc<DiscoveryManager>,
}

impl MeshProxy {
    /// Create a new mesh proxy
    pub async fn new(config: ProxyConfig) -> Result<Self> {
        info!("Starting mesh proxy for cluster: {}", config.cluster_id);

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

        // Create iroh endpoint
        let endpoint = Endpoint::builder()
            .secret_key(secret_key)
            .bind()
            .await
            .map_err(|e| Error::Network(format!("Failed to bind endpoint: {}", e)))?;

        info!("Proxy endpoint created with NodeId: {}", endpoint.node_id());

        // Create discovery manager
        let discovery = Arc::new(DiscoveryManager::new(config.cluster_id.clone()).await?);

        Ok(Self {
            config,
            endpoint,
            discovery,
        })
    }

    /// Start the proxy server
    pub async fn run(&self) -> Result<()> {
        info!("Starting proxy server on {}", self.config.bind_address);

        // Start the TCP proxy listener
        let proxy_task = self.start_tcp_proxy();

        // Start the iroh protocol handler
        let protocol_task = self.start_protocol_handler();

        // Run both tasks concurrently
        tokio::select! {
            result = proxy_task => {
                error!("TCP proxy task ended: {:?}", result);
                result
            }
            result = protocol_task => {
                error!("Protocol handler task ended: {:?}", result);
                result
            }
        }
    }

    /// Start the TCP proxy listener
    async fn start_tcp_proxy(&self) -> Result<()> {
        let listener = TcpListener::bind(self.config.bind_address).await?;
        info!("TCP proxy listening on {}", self.config.bind_address);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("Accepted connection from {}", addr);
                    let proxy = Arc::new(self.clone());
                    tokio::spawn(async move {
                        if let Err(e) = proxy.handle_tcp_connection(stream).await {
                            warn!("Error handling connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Start the iroh protocol handler
    async fn start_protocol_handler(&self) -> Result<()> {
        let handler = MeshProtocolHandler {
            discovery: self.discovery.clone(),
        };

        let router = iroh::protocol::Router::builder(self.endpoint.clone())
            .accept(MESH_ALPN, handler)
            .spawn();

        // Keep the router alive
        tokio::signal::ctrl_c().await
            .map_err(|e| Error::Network(format!("Signal handling error: {}", e)))?;

        router.shutdown().await
            .map_err(|e| Error::Network(format!("Router shutdown error: {}", e)))?;

        Ok(())
    }

    /// Handle incoming TCP connection
    async fn handle_tcp_connection(&self, mut stream: TcpStream) -> Result<()> {
        // Read the destination from the connection
        // For now, this is a simplified implementation
        // In practice, you'd parse HTTP headers or use iptables REDIRECT rules
        
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;
        
        if n == 0 {
            return Ok(());
        }

        // Simple HTTP header parsing to extract Host header
        let request = String::from_utf8_lossy(&buffer[..n]);
        let host = self.extract_host_header(&request)?;
        
        debug!("Extracted host: {}", host);

        // For demo purposes, echo back the request
        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello, Mesh!").await?;
        
        Ok(())
    }

    /// Extract host header from HTTP request
    fn extract_host_header(&self, request: &str) -> Result<String> {
        for line in request.lines() {
            if line.to_lowercase().starts_with("host:") {
                if let Some(host) = line.splitn(2, ':').nth(1) {
                    return Ok(host.trim().to_string());
                }
            }
        }
        Err(Error::Proxy("No Host header found".to_string()))
    }

    /// Get the NodeAddr for this proxy
    pub async fn node_addr(&self) -> Option<NodeAddr> {
        use n0_watcher::Watcher;
        self.endpoint.node_addr().get()
    }

    /// Get the NodeId for this proxy
    pub fn node_id(&self) -> NodeId {
        self.endpoint.node_id()
    }
}

impl Clone for MeshProxy {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            endpoint: self.endpoint.clone(),
            discovery: self.discovery.clone(),
        }
    }
}

/// Protocol handler for mesh communication
#[derive(Debug, Clone)]
struct MeshProtocolHandler {
    discovery: Arc<DiscoveryManager>,
}

impl ProtocolHandler for MeshProtocolHandler {
    async fn accept(&self, connection: iroh::endpoint::Connection) -> std::result::Result<(), iroh::protocol::AcceptError> {
        let remote_node_id = connection.remote_node_id()?;
        info!("Accepted mesh connection from {}", remote_node_id);

        // Accept a bidirectional stream
        let (mut send, mut recv) = connection.accept_bi().await?;

        // Echo data back for now
        let bytes_copied = tokio::io::copy(&mut recv, &mut send).await?;
        debug!("Copied {} bytes", bytes_copied);

        send.finish()?;
        connection.closed().await;

        Ok(())
    }
}