# Iroh Mesh Architecture

This document provides detailed architectural diagrams and explanations for the Iroh Mesh service mesh implementation.

## System Overview

```mermaid
graph TB
    subgraph "Cluster A"
        A1[App 1] --> AP[Iroh Proxy A]
        A2[App 2] --> AP
        AA[Iroh Agent A] --> K8sA[Kubernetes API A]
        AP --> AA
    end
    
    subgraph "Cluster B"
        B1[App 3] --> BP[Iroh Proxy B]
        B2[App 4] --> BP
        BA[Iroh Agent B] --> K8sB[Kubernetes API B]
        BP --> BA
    end
    
    subgraph "P2P Network"
        AP <--> IROH[Iroh P2P Connection<br/>QUIC + Hole Punching]
        BP <--> IROH
        AA <--> IROH
        BA <--> IROH
    end
    
    subgraph "Relay Network"
        IROH --> R1[Relay Server 1]
        IROH --> R2[Relay Server 2]
        IROH --> R3[Relay Server N]
    end
```

## Component Architecture

### Mesh Proxy Architecture

```mermaid
graph TB
    subgraph "Mesh Proxy"
        TI[Traffic Interceptor] --> PP[Protocol Parser]
        PP --> RT[Route Table]
        RT --> CP[Connection Pool]
        CP --> IE[Iroh Endpoint]
        
        subgraph "Traffic Interceptor"
            HTTP[HTTP Proxy]
            IPTABLES[iptables REDIRECT]
            SIDECAR[Sidecar Injection]
        end
        
        subgraph "Connection Pool"
            CC[Cluster Connections]
            HC[Health Checking]
            LB[Load Balancing]
        end
    end
    
    APP[Application] --> TI
    IE --> REMOTE[Remote Cluster]
    DM[Discovery Manager] --> RT
```

### Mesh Agent Architecture

```mermaid
graph TB
    subgraph "Mesh Agent"
        API[HTTP API Server] --> DM[Discovery Manager]
        DM --> KS[Kubernetes Scanner]
        DM --> CR[Cluster Registry]
        DM --> IE[Iroh Endpoint]
        
        subgraph "Discovery Manager"
            LS[Local Services]
            RS[Remote Services]
            CM[Cluster Mapping]
        end
        
        subgraph "Kubernetes Scanner"
            SA[Service API]
            EA[Endpoints API]
            PA[Pod API]
        end
    end
    
    K8S[Kubernetes API] --> KS
    IE --> PEERS[Peer Clusters]
    MONITORING[Monitoring] --> API
```

## Traffic Flow Diagrams

### Intra-Cluster Communication

```mermaid
sequenceDiagram
    participant A1 as App 1
    participant A2 as App 2
    participant K8s as Kubernetes
    
    A1->>K8s: DNS Lookup (service-b.namespace.svc.cluster.local)
    K8s-->>A1: Pod IP
    A1->>A2: Direct HTTP Request
    A2-->>A1: HTTP Response
    
    Note over A1,A2: Standard Kubernetes networking
```

### Cross-Cluster Communication

```mermaid
sequenceDiagram
    participant A1 as App 1 (Cluster A)
    participant PA as Proxy A
    participant AA as Agent A
    participant AB as Agent B
    participant PB as Proxy B
    participant B1 as App 2 (Cluster B)
    
    A1->>PA: HTTP Request to service-b.cluster-b
    PA->>AA: Lookup service-b in cluster-b
    AA->>AB: Discovery Query (via iroh)
    AB-->>AA: Service Location
    AA-->>PA: Target: Proxy B
    PA->>PB: HTTP-over-QUIC (via iroh)
    PB->>B1: Local HTTP Request
    B1-->>PB: HTTP Response
    PB-->>PA: HTTP-over-QUIC Response
    PA-->>A1: HTTP Response
```

## Connection Establishment

### Initial Cluster Discovery

```mermaid
sequenceDiagram
    participant CA as Cluster A Agent
    participant CB as Cluster B Agent
    participant DNS as DNS Discovery
    participant RELAY as Relay Server
    
    Note over CA,CB: Bootstrap Phase
    CA->>DNS: Publish cluster-a.mesh NodeAddr
    CB->>DNS: Publish cluster-b.mesh NodeAddr
    
    Note over CA,CB: Discovery Phase
    CA->>DNS: Query cluster-b.mesh
    DNS-->>CA: NodeAddr for Cluster B
    CA->>RELAY: Connect to Cluster B (via relay)
    RELAY->>CB: Forward connection
    CB-->>CA: Accept connection
    
    Note over CA,CB: Direct Connection Phase
    CA->>CB: Hole Punching Attempt
    CA->>CB: Direct QUIC Connection Established
```

### Service Discovery Protocol

```mermaid
sequenceDiagram
    participant PA as Proxy A
    participant AA as Agent A
    participant AB as Agent B
    participant K8s as Kubernetes B
    
    PA->>AA: Resolve "payment-service.cluster-b"
    
    alt Cache Hit
        AA-->>PA: Cached Service Info
    else Cache Miss
        AA->>AB: ServiceDiscoveryQuery{service: "payment-service"}
        AB->>K8s: List Services(label: app=payment-service)
        K8s-->>AB: Service Details + Endpoints
        AB-->>AA: ServiceDiscoveryResponse{endpoints, ports, metadata}
        AA->>AA: Cache Response
        AA-->>PA: Service Info
    end
```

## Data Structures

### Core Types

```mermaid
classDiagram
    class ClusterId {
        +String id
    }
    
    class ClusterInfo {
        +ClusterId id
        +NodeId node_id
        +Option~String~ relay_url
        +Vec~SocketAddr~ direct_addresses
        +Vec~ServiceInfo~ services
    }
    
    class ServiceInfo {
        +String name
        +String namespace
        +u16 port
        +String protocol
    }
    
    class CrossClusterRoute {
        +ClusterId target_cluster
        +String target_service
        +String target_namespace
        +u16 target_port
    }
    
    ClusterInfo --> ClusterId
    ClusterInfo --> ServiceInfo
    CrossClusterRoute --> ClusterId
```

### Configuration Hierarchy

```mermaid
graph TB
    subgraph "Configuration"
        PC[ProxyConfig] --> KC[KubernetesConfig]
        AC[AgentConfig] --> KC
        AC --> DC[DiscoveryConfig]
        
        PC --> BPA[bind_address]
        PC --> CID[cluster_id]
        PC --> SKP[secret_key_path]
        PC --> EI[enable_interception]
        
        DC --> EDN[enable_dns]
        DC --> EMD[enable_mdns]
        DC --> EP[endpoints]
    end
```

## Deployment Models

### Sidecar Deployment

```mermaid
graph TB
    subgraph "Kubernetes Pod"
        subgraph "App Container"
            APP[Application]
        end
        
        subgraph "Proxy Sidecar"
            PROXY[Iroh Proxy]
        end
        
        APP --> PROXY
    end
    
    subgraph "Node"
        subgraph "Agent DaemonSet"
            AGENT[Iroh Agent]
        end
    end
    
    PROXY --> AGENT
    AGENT --> K8S[Kubernetes API]
```

### Node-Level Deployment

```mermaid
graph TB
    subgraph "Node 1"
        subgraph "Pod A"
            A1[App 1]
        end
        subgraph "Pod B"
            A2[App 2]
        end
        
        NP1[Node Proxy] --> A1
        NP1 --> A2
        NA1[Node Agent]
    end
    
    subgraph "Node 2"
        subgraph "Pod C"
            A3[App 3]
        end
        
        NP2[Node Proxy] --> A3
        NA2[Node Agent]
    end
    
    NP1 <--> NP2
    NA1 <--> NA2
```

## Security Architecture

### Authentication & Authorization

```mermaid
graph TB
    subgraph "Cluster A"
        CA[Cluster A] --> SKA[Secret Key A]
        SKA --> PKA[Public Key A / NodeId A]
    end
    
    subgraph "Cluster B"
        CB[Cluster B] --> SKB[Secret Key B]
        SKB --> PKB[Public Key B / NodeId B]
    end
    
    subgraph "Connection Security"
        PKA --> TLS[mTLS Connection]
        PKB --> TLS
        TLS --> QUIC[QUIC Encryption]
    end
    
    subgraph "Service Authorization"
        POLICY[Service Policies] --> RBAC[Cluster RBAC]
        RBAC --> ALLOW[Allow/Deny]
    end
```

### Traffic Encryption

```mermaid
sequenceDiagram
    participant A as App A
    participant PA as Proxy A
    participant PB as Proxy B
    participant B as App B
    
    Note over A,B: Clear HTTP within cluster
    A->>PA: HTTP Request (cleartext)
    
    Note over PA,PB: Encrypted transport between clusters
    PA->>PB: QUIC/TLS 1.3 (encrypted)
    
    Note over PB,B: Clear HTTP within cluster
    PB->>B: HTTP Request (cleartext)
    B-->>PB: HTTP Response (cleartext)
    
    Note over PA,PB: Encrypted transport between clusters  
    PB-->>PA: QUIC/TLS 1.3 (encrypted)
    
    Note over A,PA: Clear HTTP within cluster
    PA-->>A: HTTP Response (cleartext)
```

## Performance Characteristics

### Connection Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Discovering: Cluster Discovery
    Discovering --> Connecting: NodeAddr Found
    Connecting --> Relayed: Direct Failed
    Connecting --> Direct: Hole Punch Success
    Relayed --> Direct: Upgrade Available
    Direct --> [*]: Connection Closed
    Relayed --> [*]: Connection Closed
    
    Direct: Direct QUIC Connection
    Relayed: Relay-Mediated Connection
```

### Load Balancing Strategy

```mermaid
graph TB
    subgraph "Load Balancing"
        REQ[Incoming Request] --> LB[Load Balancer]
        LB --> ALG[Algorithm]
        
        subgraph "Algorithms"
            RR[Round Robin]
            LC[Least Connections]
            WRR[Weighted Round Robin]
            EWMA[EWMA Latency]
        end
        
        ALG --> RR
        ALG --> LC
        ALG --> WRR
        ALG --> EWMA
    end
    
    subgraph "Target Selection"
        LB --> C1[Cluster 1]
        LB --> C2[Cluster 2]
        LB --> C3[Cluster N]
        
        C1 --> S1[Service Instance 1]
        C1 --> S2[Service Instance 2]
    end
```

## Error Handling & Resilience

### Circuit Breaker Pattern

```mermaid
stateDiagram-v2
    [*] --> Closed: Initial State
    Closed --> Open: Failure Threshold
    Open --> HalfOpen: Timeout Elapsed
    HalfOpen --> Closed: Success
    HalfOpen --> Open: Failure
    
    Closed: Allow Traffic
    Open: Reject Traffic
    HalfOpen: Test Connection
```

### Retry Strategy

```mermaid
sequenceDiagram
    participant C as Client
    participant P as Proxy
    participant T as Target
    
    C->>P: Request
    P->>T: Attempt 1
    T--xP: Connection Failed
    
    Note over P: Exponential Backoff
    P->>T: Attempt 2 (after delay)
    T--xP: Connection Failed
    
    Note over P: Exponential Backoff  
    P->>T: Attempt 3 (after longer delay)
    T-->>P: Success
    P-->>C: Response
```

This architecture provides a comprehensive foundation for building a production-ready P2P service mesh using iroh's networking capabilities while maintaining compatibility with existing Kubernetes infrastructure.