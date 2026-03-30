# =============================================================================
# Stage 1: Build
# =============================================================================
FROM ubuntu:24.04 AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    build-essential \
    libavif-dev \
    libdav1d-dev \
    libssl-dev \
    pkg-config \
    libclang-dev \
    nasm \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain 1.94.0 --no-modify-path
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

RUN cargo build --release -p app

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM ubuntu:24.04 AS runtime-app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libdav1d7 \
    libssl3t64 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/app /app/app
COPY config/ /app/config/
COPY app/assets/ /app/app/assets/

RUN mkdir -p /app/logs

EXPOSE 3001

ENV RUN_ENV=production \
    RUST_LOG=info

ENTRYPOINT ["/app/app"]
