# Pinned to linux/amd64: candle's gemm backend fails to build on linux/arm64 (fullfp16 asm).
FROM --platform=linux/amd64 rust:bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY rust-toolchain.toml ./

# Copy the full workspace (root Cargo.toml declares members = ["bins/*", "crates/*"],
# so every member's manifest must be present for `cargo build` to parse the workspace)
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY bins ./bins

# Build release binary
RUN cargo build --release --package executor

# Runtime stage
FROM --platform=linux/amd64 debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/executor /app/executor

CMD ["/app/executor"]
