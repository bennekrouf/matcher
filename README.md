# API Matcher

A gRPC service that matches natural language queries to API endpoints using embeddings and vector similarity search.

## Features

[Previous features section remains the same...]

## Architecture

[Previous architecture section remains the same...]

## Prerequisites

- Rust 1.70 or higher
- curl (for downloading models)
- gRPCurl (for testing gRPC endpoints)
- Docker (for server and development modes)
- At least 4GB of free disk space for models and Docker images

## Operation Modes

### 1. Standalone Mode

[Previous standalone mode section remains the same...]

### 2. Docker Server Mode

#### Basic Docker Operations
```bash
# Build Docker image
docker build -t api-matcher .

# Run container
docker run -d \
  --name api-matcher \
  -p 50030:50030 \
  -v api-matcher-data:/app/data \
  -v $(pwd)/logs:/app/logs \
  api-matcher

# Stop container
docker stop api-matcher

# Remove container
docker rm api-matcher

# Remove image
docker rmi api-matcher
```

#### Docker Management
```bash
# Stop all running containers
docker stop $(docker ps -a -q)

# Remove all stopped containers
docker rm $(docker ps -a -q)

# Clean up system (remove unused containers, networks, images)
docker system prune

# Remove everything (use with caution)
docker system prune -a
```

#### Logging
```bash
# View real-time logs
docker logs -f api-matcher

# Show last 100 lines with timestamps
docker logs -f -t --tail 100 api-matcher

# Save logs to file
docker logs api-matcher > matcher_logs.txt
```

#### Volume Management
```bash
# Run with persistent data and logs
docker run -d \
  -p 50030:50030 \
  -v api-matcher-data:/app/data \
  -v $(pwd)/logs:/app/logs \
  --name api-matcher \
  api-matcher

# Check volume status
docker volume ls
```

#### Debugging
```bash
# Check container status
docker ps
docker stats api-matcher

# Interactive shell access
docker exec -it api-matcher /bin/bash

# Check resource usage
docker stats
```

#### Environment Variables
- `MODEL_VERSION`: Specify model version (default: paraphrase-multilingual-MiniLM-L12-v2)
- `CONFIG_PATH`: Custom config file path (default: /app/endpoints.yaml)
- `MODEL_PATH`: Custom model path (default: /app/models/multilingual-MiniLM)
- `LOG_LEVEL`: Set logging level (default: info)

#### Volumes
- `/app/data`: Vector database storage
- `/app/logs`: Application logs
- `/app/config`: Configuration files

### 3. Development Mode

#### With Auto-Reload
```bash
# Start development environment
docker compose -f docker-compose.dev.yml up -d

# View logs in development
docker compose -f docker-compose.dev.yml logs -f matcher

# Stop development environment
docker compose -f docker-compose.dev.yml down
```

#### Development Docker Compose
```yaml
version: '3.8'
services:
  matcher:
    build: 
      context: .
      target: development
    volumes:
      - .:/usr/src/app
      - /usr/src/app/target
    ports:
      - "50030:50030"
    environment:
      - RUST_LOG=debug
    command: cargo watch -x run -- --server
```

## Troubleshooting

### Common Docker Issues

1. **Container won't start**
```bash
# Check logs for errors
docker logs api-matcher

# Verify port availability
lsof -i :50030

# Check container status
docker ps -a
```

2. **Performance Issues**
```bash
# Monitor resource usage
docker stats api-matcher

# Check available disk space
docker system df
```

3. **Clean Up Resources**
```bash
# Remove unused resources
docker system prune

# Remove specific container and image
docker stop api-matcher && \
docker rm api-matcher && \
docker rmi api-matcher
```

### Model Download Issues

```bash
# Manually download models
docker exec -it api-matcher bash
curl -L [model-url] -o [model-path]

# Verify model files
docker exec api-matcher ls -l /app/models/multilingual-MiniLM
```

## Production Deployment

### Resource Requirements
- Minimum 2 CPU cores
- 4GB RAM
- 10GB disk space
- Docker 20.10 or newer

### Security Considerations
- Run container as non-root user
- Use Docker secrets for sensitive data
- Enable Docker content trust
- Regular security updates

### Monitoring
```bash
# Basic health check
docker inspect api-matcher

# Resource monitoring
docker stats api-matcher

# Log monitoring
docker logs -f api-matcher
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

