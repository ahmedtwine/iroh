# Iroh Mesh

A P2P service mesh for Kubernetes using [iroh](https://github.com/n0-computer/iroh) for cross-cluster communication.

## Overview

Iroh Mesh enables direct, encrypted peer-to-peer connections between Kubernetes clusters using iroh's hole-punching and relay capabilities. It provides a service mesh architecture specifically designed for multi-cluster deployments where clusters may be behind NATs or firewalls.

## Architecture

The system consists of two main components:

### 1. Mesh Proxy (`iroh-proxy`)
- **Purpose**: Intercepts outbound traffic and routes cross-cluster requests through iroh connections
- **Deployment**: Sidecar or node-level proxy
- **Features**:
  - Traffic interception (HTTP/TCP)
  - Protocol translation (HTTP over iroh QUIC streams)
  - Connection pooling and management
  - Load balancing across clusters

### 2. Mesh Agent (`iroh-agent`)
- **Purpose**: Manages cluster coordination and service discovery
- **Deployment**: DaemonSet on cluster nodes
- **Features**:
  - Cross-cluster service discovery
  - Cluster registration and heartbeat
  - Node and service information sharing
  - RESTful API for status and configuration

## Core Concepts

### Cluster Identity
Each cluster has a unique `ClusterId` and iroh `NodeId` for identification and authentication.

### Service Discovery
Cross-cluster service discovery allows services in one cluster to discover and communicate with services in remote clusters.

### Traffic Flow
```
App → Mesh Proxy → Iroh Connection → Remote Proxy → Remote Service
```

## Getting Started

### Prerequisites
- Rust 1.85+
- Kubernetes cluster
- `just` command runner (optional)

### Building

```bash
# Using just (recommended)
just mesh-check
just mesh-build

# Or directly with cargo
cd iroh-mesh
K8S_OPENAPI_ENABLED_VERSION=1.30 cargo build
```

### Running Components

#### Start the Mesh Agent
```bash
just mesh-agent
# OR
cd iroh-mesh && K8S_OPENAPI_ENABLED_VERSION=1.30 cargo run --bin iroh-agent
```

#### Start the Mesh Proxy
```bash
just mesh-proxy
# OR
cd iroh-mesh && K8S_OPENAPI_ENABLED_VERSION=1.30 cargo run --bin iroh-proxy
```

#### Demo Mode (Both Components)
```bash
just mesh-demo
```

### Configuration

Both components support configuration via CLI arguments or configuration files:

#### Proxy Configuration
```bash
iroh-proxy --cluster-id my-cluster --bind 127.0.0.1:15001
```

#### Agent Configuration
```bash
iroh-agent --cluster-id my-cluster --bind 127.0.0.1:15002 --enable-dns
```

## Development

### Project Structure
```
iroh-mesh/
├── src/
│   ├── lib.rs              # Core types and constants
│   ├── error.rs            # Error handling
│   ├── config.rs           # Configuration management
│   ├── proxy.rs            # Mesh proxy implementation
│   ├── agent.rs            # Mesh agent implementation
│   ├── discovery.rs        # Service discovery logic
│   └── bin/
│       ├── proxy.rs        # Proxy binary
│       └── agent.rs        # Agent binary
├── Cargo.toml
└── README.md
```

### Key Dependencies
- **iroh**: P2P networking and QUIC transport
- **kube**: Kubernetes client library
- **tokio**: Async runtime
- **tower**: Service abstractions (future integration)
- **hyper**: HTTP client/server (planned)

### Available Commands

```bash
# Development
just mesh-check           # Type check
just mesh-build           # Build binaries
just mesh-test            # Run tests
just mesh-docs            # Generate documentation

# Runtime
just mesh-proxy           # Run proxy
just mesh-agent           # Run agent
just mesh-demo            # Run both in tmux
```

## Architecture Deep Dive

### Traffic Interception

The mesh proxy intercepts outbound traffic using various mechanisms:
- **iptables REDIRECT**: Transparent proxy mode (future)
- **HTTP proxy**: Application-configured proxy mode
- **Sidecar injection**: Kubernetes sidecar pattern

### Connection Management

Each cluster maintains:
- **Iroh Endpoint**: QUIC endpoint for P2P connections
- **Node Map**: Mapping of cluster IDs to iroh NodeAddrs
- **Service Registry**: Local and remote service information
- **Connection Pool**: Persistent connections to remote clusters

### Service Discovery

Multi-layered discovery system:
1. **Local Discovery**: Kubernetes API for local services
2. **Cross-Cluster Discovery**: Custom protocol over iroh connections
3. **DNS Integration**: Optional DNS-based service resolution
4. **Static Configuration**: Manual cluster and service configuration

### Security Model

- **Identity**: Each cluster has a unique iroh SecretKey
- **Authentication**: Automatic mTLS through iroh QUIC
- **Authorization**: Cluster-level and service-level access control
- **Encryption**: All traffic encrypted by default via QUIC

## Future Roadmap

### Phase 1: Foundation ✓
- [x] Basic proxy and agent structure
- [x] Iroh integration
- [x] Kubernetes client setup
- [x] Configuration management

### Phase 2: Core Functionality (Next)
- [ ] Traffic interception implementation
- [ ] Cross-cluster service discovery
- [ ] Basic HTTP-over-iroh protocol
- [ ] Connection pooling

### Phase 3: Production Features
- [ ] eBPF traffic interception
- [ ] Advanced load balancing
- [ ] Observability and metrics
- [ ] Security policies
- [ ] Multi-protocol support

### Phase 4: Advanced Features
- [ ] Service mesh integration (Linkerd/Istio compatibility)
- [ ] Advanced traffic management
- [ ] Circuit breaking and retries
- [ ] Distributed tracing

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the same terms as iroh - either MIT or Apache 2.0, at your option.