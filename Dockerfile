FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    tini \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Create directories
RUN mkdir -p /app/models/multilingual-MiniLM /app/data/mydb /app/logs

# Copy the binary and make it executable
COPY target/release/matcher /app/matcher
RUN chmod +x /app/matcher

# Copy config
COPY config/endpoints.yaml /app/endpoints.yaml

# Download model files
RUN curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/config.json -o /app/models/multilingual-MiniLM/config.json && \
    curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/tokenizer.json -o /app/models/multilingual-MiniLM/tokenizer.json && \
    curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/pytorch_model.bin -o /app/models/multilingual-MiniLM/model.ot

RUN chmod -R 777 /app/data /app/logs

COPY docker-entrypoint.sh /app/
RUN chmod +x /app/docker-entrypoint.sh

EXPOSE 50030
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/app/docker-entrypoint.sh"]
