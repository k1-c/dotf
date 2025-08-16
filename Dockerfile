# Multi-stage build for optimized binary
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    git \
    openssl-dev \
    openssl-libs-static

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual binary
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache \
    git \
    ca-certificates \
    && addgroup -g 1000 dott \
    && adduser -D -s /bin/sh -u 1000 -G dott dott

# Copy the binary
COPY --from=builder /app/target/release/dott /usr/local/bin/dott

# Create dott user and directories
USER dott
WORKDIR /home/dott

# Set environment variables
ENV RUST_LOG=info
ENV DOTT_HOME=/home/dott/.dott

# Create .dott directory
RUN mkdir -p $DOTT_HOME

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD dott --version || exit 1

ENTRYPOINT ["dott"]
CMD ["--help"]