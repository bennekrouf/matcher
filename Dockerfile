# Builder stage
FROM rust:1.81-slim as builder
# Install build dependencies including C++
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    build-essential \
    g++ \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies and logging utilities
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    tini \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create necessary directories including logs
RUN mkdir -p /app/models/multilingual-MiniLM /app/data/mydb /app/logs

# Copy the executable and config
COPY --from=builder /usr/src/app/target/release/matcher /app/matcher
COPY config/endpoints.yaml /app/endpoints.yaml

# Download model files during container build
RUN curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/config.json -o /app/models/multilingual-MiniLM/config.json && \
    curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/tokenizer.json -o /app/models/multilingual-MiniLM/tokenizer.json && \
    curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/pytorch_model.bin -o /app/models/multilingual-MiniLM/model.ot

# Make directories writable
RUN chmod -R 777 /app/data /app/logs

EXPOSE 50030

# Copy initialization script
COPY docker-entrypoint.sh /app/
RUN chmod +x /app/docker-entrypoint.sh

# Use tini as init system
ENTRYPOINT ["/usr/bin/tini", "--"]

# Use the initialization script
CMD ["/app/docker-entrypoint.sh"]
