
# API Matcher 

A gRPC service that matches natural language queries to API endpoints using embeddings and vector similarity search.

## Features

- Natural language processing for API endpoint matching
- Multilingual support (English and French)
- Interactive parameter collection through gRPC streaming
- Vector similarity search using LanceDB
- Embedding-based matching using Sentence Transformers
- CLI and Server modes
- Bi-directional streaming for interactive queries

## Architecture

- **Rust Backend**: High-performance gRPC server
- **Embeddings**: sentence-transformers for semantic understanding
- **Vector DB**: LanceDB for efficient similarity search
- **Protocol**: gRPC/Protocol Buffers for client-server communication

## Prerequisites

- Rust 1.70 or higher
- curl (for downloading models)
- gRPCurl (for testing gRPC endpoints)
- Docker (optional, for Envoy proxy in development)

## Installation

### 1. Clone the Repository

```bash
git clone [your-repo-url]
cd api-matcher
```

### 2. Download the Embedding Model

Choose one of these options:

```bash
# Option A: English-optimized model (smaller)
mkdir -p models/all-MiniLM-L6-v2
curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/config.json -o models/all-MiniLM-L6-v2/config.json
curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json -o models/all-MiniLM-L6-v2/tokenizer.json
curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/pytorch_model.bin -o models/all-MiniLM-L6-v2/model.ot
```

```bash
# Option B: Multilingual model (recommended)
mkdir -p models/multilingual-MiniLM
curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/config.json -o models/multilingual-MiniLM/config.json
curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/tokenizer.json -o models/multilingual-MiniLM/tokenizer.json
curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/pytorch_model.bin -o models/multilingual-MiniLM/model.ot
```

### 3. Build the Project

```bash
cargo build
```

## Usage

### CLI Mode

The matcher can be used directly from the command line:

```bash
# Show help
matcher --help

# Show version
matcher --version

# Basic English query
matcher -q "run analysis"

# French query with language specification
matcher -q "lancer l'analyse" --language fr

# Verbose output with multiple results
matcher -q "run analysis" -v --limit 3

# Reload database and run query
matcher --reload -q "run analysis" -v
```

### Server Mode

#### Start the Server

```bash
matcher --server
```

#### Development Setup with Envoy

1. Create Envoy configuration:

```bash
# envoy/envoy.yaml
static_resources:
  listeners:
    - name: listener_0
      address:
        socket_address: { address: 0.0.0.0, port_value: 9090 }
      filter_chains:
        - filters:
            - name: envoy.filters.network.http_connection_manager
              typed_config:
                "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
                codec_type: auto
                stat_prefix: ingress_http
                route_config:
                  name: local_route
                  virtual_hosts:
                    - name: local_service
                      domains: ["*"]
                      routes:
                        - match: { prefix: "/" }
                          route:
                            cluster: matcher_service
                            timeout: 0s
                            max_stream_duration:
                              grpc_timeout_header_max: 0s
                http_filters:
                  - name: envoy.filters.http.grpc_web
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.grpc_web.v3.GrpcWeb
                  - name: envoy.filters.http.cors
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.cors.v3.Cors
                  - name: envoy.filters.http.router
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.router.v3.Router
  clusters:
    - name: matcher_service
      type: STRICT_DNS
      dns_lookup_family: V4_ONLY
      load_assignment:
        cluster_name: matcher_service
        endpoints:
          - lb_endpoints:
              - endpoint:
                  address:
                    socket_address:
                      address: host.docker.internal
                      port_value: 50030
```

2. Start Envoy proxy:

```bash
docker-compose up -d
```

#### Testing gRPC Endpoints

Using grpcurl for interactive testing:

```bash
# Start interactive session
grpcurl -d @ -plaintext localhost:50030 matcher.Matcher/InteractiveMatch

# Send initial query
{"initial_query": {"query": "send email", "language": "fr"}}

# Send confirmation when prompted
{"confirmation_response": {"confirmed": true}}

# Send parameter values when prompted
{"parameter_value": {"parameter_name": "email", "value": "test@example.com"}}
```

### Development Commands

```bash
# Reload database
cargo run -- --reload

# Test query with reload
cargo run -- --reload --query "run the best analysis"

# Debug mode with all matches
cargo run -- --reload --debug --all --query "run an analysis"
```


#### Start dev environment with docker compose to have auto refresh:
```bash

# Start development environment
docker compose -f docker-compose.dev.yml up -d

# View logs
docker compose -f docker-compose.dev.yml logs -f matcher

# When done
docker compose -f docker-compose.dev.yml down
```

## Vector Database Structure

The project uses LanceDB for storing and searching endpoint embeddings:

- Each record contains:
  - Endpoint ID
  - Vector embedding (384 dimensions for MiniLM)
  - Parameter definitions
  - Language specification
  - Example queries

## Configuration

Key configuration options in `config.yml`:

```yaml
model:
  path: "models/multilingual-MiniLM"
  dimension: 384

database:
  path: "data/mydb"
  table: "endpoints"

server:
  host: "0.0.0.0"
  port: 50030
```

## Testing

Run the test suite:

```bash
cargo test

# Run interactive tests
cd tests/interactive
cargo run
```

## Docker Support

```bash
# Build the image
docker build -t api-matcher .

# Run the container
docker run -p 50030:50030 api-matcher
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [sentence-transformers](https://github.com/UKPLab/sentence-transformers) for the embedding models
- [LanceDB](https://github.com/lancedb/lancedb) for the vector database
- [tonic](https://github.com/hyperium/tonic) for the gRPC implementation

