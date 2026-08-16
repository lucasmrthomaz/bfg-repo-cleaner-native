# Build stage
FROM rust:bookworm AS builder

WORKDIR /usr/src/bfg

# Install build dependencies for git2 / openssl
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY src-tauri ./src-tauri

RUN cargo build --release --bin bfg-repo-cleaner-native

# Runtime stage
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    git \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/bfg/target/release/bfg-repo-cleaner-native /usr/local/bin/bfg-repo-cleaner-native

WORKDIR /repo

VOLUME ["/repo"]

ENTRYPOINT ["bfg-repo-cleaner-native"]
CMD ["--help"]
