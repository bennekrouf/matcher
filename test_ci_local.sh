#!/bin/bash
# test_ci_local.sh

# Create staging directory
mkdir -p docker_build

# Copy binary and other files
cp target/release/matcher docker_build/
cp config/endpoints.yaml docker_build/
cp docker-entrypoint.sh docker_build/
chmod +x docker_build/matcher

# Create Dockerfile in staging directory
cat >docker_build/Dockerfile <<'EOF'
FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    tini \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN mkdir -p /app/models/multilingual-MiniLM /app/data/mydb /app/logs
COPY matcher /app/matcher
COPY endpoints.yaml /app/endpoints.yaml
COPY docker-entrypoint.sh /app/
RUN chmod +x /app/matcher /app/docker-entrypoint.sh && \
    curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/config.json -o /app/models/multilingual-MiniLM/config.json && \
    curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/tokenizer.json -o /app/models/multilingual-MiniLM/tokenizer.json && \
    curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/pytorch_model.bin -o /app/models/multilingual-MiniLM/model.ot && \
    chmod -R 777 /app/data /app/logs
EXPOSE 50030
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/docker-entrypoint.sh"]
EOF

# Build from staging directory
cd docker_build
docker build -t api-matcher:local .
cd ..

# Start containers
export BRANCH_NAME=local
docker-compose -f docker-compose.test.yml up -d

# Wait for services to be ready
echo "Waiting for services to start..."
sleep 10

# Run tests
cd tests/interactive
cargo run -- --test

# Show logs if tests fail
if [ $? -ne 0 ]; then
  docker-compose -f ../../docker-compose.test.yml logs
fi

# Cleanup
cd ../..
docker-compose -f docker-compose.test.yml down
