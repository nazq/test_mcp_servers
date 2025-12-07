FROM rust:1.91-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source and rebuild
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM alpine:3.21

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/mcp-test-server /usr/local/bin/

ENV MCP_PORT=3000 \
    MCP_HOST=0.0.0.0 \
    MCP_LOG_LEVEL=info

EXPOSE 3000

ENTRYPOINT ["mcp-test-server"]
