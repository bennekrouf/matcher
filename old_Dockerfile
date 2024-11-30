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
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create directories
RUN mkdir -p /app/models/multilingual-MiniLM /app/data/mydb

# Copy the executable
COPY --from=builder /usr/src/app/target/release/matcher /app/matcher

# Copy the config file to root level as per CONFIG_PATH
COPY config/endpoints.yaml /app/endpoints.yaml

# Copy model files to exact MODEL_PATH location
COPY models/multilingual-MiniLM/config.json /app/models/multilingual-MiniLM/
COPY models/multilingual-MiniLM/model.ot /app/models/multilingual-MiniLM/
COPY models/multilingual-MiniLM/tokenizer.json /app/models/multilingual-MiniLM/

# Debug: Print directory structure
RUN echo "Directory structure:" && \
    ls -R /app && \
    echo "Model files:" && \
    ls -l /app/models/multilingual-MiniLM/ && \
    echo "Config file:" && \
    ls -l /app/endpoints.yaml

# Make data directory writable
RUN chmod -R 777 /app/data

EXPOSE 50030

# Run with server mode
CMD ["./matcher", "--server"]
