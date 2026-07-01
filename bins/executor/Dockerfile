# Build stage
# glibc base: executor depends on the `cortex` crate (tch / rust-bert / ort),
# which needs libtorch + ONNX runtime and cannot build on musl/alpine.
FROM rust:bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev unzip curl \
    && rm -rf /var/lib/apt/lists/*

# torch-sys 0.17 only auto-downloads x86_64 CPU libtorch, which won't link on an
# aarch64 build (Apple Silicon). Instead we provide a native libtorch matching
# the build's processor arch, extracted from PyTorch's wheel (a plain zip), and
# point LIBTORCH at it so torch-sys skips its download.
#
# The version must match torch-sys 0.17's TORCH_VERSION (2.4.0) exactly — 2.5.0's
# _scaled_mm_out C++ signature differs and fails to compile.
#
# On aarch64 the wheel splits native deps across two dirs: torch/lib holds
# libtorch*.so, while its BLAS/LAPACK (libopenblas), ARM Compute Library
# (libarm_compute*), OpenMP and gfortran runtimes live in a sibling torch.libs/.
# libtorch_cpu.so has undefined refs into those, so we flatten torch.libs into
# /opt/libtorch/lib; otherwise the final link fails with undefined references to
# ddot_/zhetrf_/arm_compute::* etc. On x86_64 the wheel has no torch.libs/, so
# the flatten step is a no-op (guarded below).
ARG LIBTORCH_VERSION=2.4.0
RUN set -eux; \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
      arm64) whl="torch-${LIBTORCH_VERSION}-cp312-cp312-manylinux_2_17_aarch64.manylinux2014_aarch64.whl" ;; \
      amd64) whl="torch-${LIBTORCH_VERSION}%2Bcpu-cp312-cp312-linux_x86_64.whl" ;; \
      *) echo "unsupported arch: $arch" >&2; exit 1 ;; \
    esac; \
    curl -fsSL -o /tmp/torch.whl "https://download.pytorch.org/whl/cpu/${whl}"; \
    unzip -q /tmp/torch.whl -d /tmp/torch; \
    mkdir -p /opt/libtorch; \
    mv /tmp/torch/torch/lib /opt/libtorch/lib; \
    mv /tmp/torch/torch/include /opt/libtorch/include; \
    if [ -d /tmp/torch/torch.libs ]; then mv /tmp/torch/torch.libs/* /opt/libtorch/lib/; fi; \
    rm -rf /tmp/torch /tmp/torch.whl
ENV LIBTORCH=/opt/libtorch
ENV LD_LIBRARY_PATH=/opt/libtorch/lib
# PyTorch's manylinux wheels are built with the pre-CXX11 std::string ABI
# (_GLIBCXX_USE_CXX11_ABI=0). torch-sys defaults its C++ glue to ABI=1 when
# LIBTORCH is set, which mangles std::string symbols differently and fails the
# final link with undefined refs to c10::detail::torchInternalAssertFail etc.
# Match the wheel's ABI.
ENV LIBTORCH_CXX11_ABI=0

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

# Copy libtorch shared libraries (executor links them dynamically) and the binary
COPY --from=builder /opt/libtorch/lib /opt/libtorch/lib
ENV LD_LIBRARY_PATH=/opt/libtorch/lib
COPY --from=builder /app/target/release/executor /app/executor

CMD ["/app/executor"]
