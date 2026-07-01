# Build stage
# glibc base: executor depends on the `cortex` crate (tch / rust-bert / ort),
# which needs libtorch + ONNX runtime and cannot build on musl/alpine.
FROM rust:bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Pin the toolchain before building so the image's rustup installs the exact
# nightly the project (and Cargo.lock) was resolved against. Without this the
# image's default stable rustc is used, which mis-builds version-sensitive deps
# (e.g. rustix 0.37's `rustc_attrs` probe).
COPY rust-toolchain.toml ./

# Copy the full workspace (root Cargo.toml declares members = ["bins/*", "crates/*"],
# so every member's manifest must be present for `cargo build` to parse the workspace)
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY bins ./bins

# Build release binary
RUN cargo build --release --package executor

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/executor /app/executor

CMD ["/app/executor"]
