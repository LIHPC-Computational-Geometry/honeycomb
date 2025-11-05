FROM rust:1.91 AS builder

WORKDIR /builder
ARG USE_CUDA=false
ENV USE_CUDA=${USE_CUDA}

# Copy files from the repo
COPY . .

# Install dependencies
RUN apt-get update && apt-get install -y libhwloc-dev libudev-dev nvidia-cuda-toolkit

RUN if [ "$INSTALL_CUDA" = "true" ]; then \
        apt-get install -y nvidia-cuda-toolkit; \
    else \ 
        echo "Skipping CUDA installation";
RUN if [ "$INSTALL_CUDA" = "true" ]; then \
        export FEATURES="--features cuda"; \
    else \ 
        export FEATURES="";

# Build binaries
RUN --mount=type=cache,target=/cargo CARGO_HOME=/cargo \
        RUSTFLAGS="-C target-cpu=native" \
        cargo install \
        --path=applications \
        --bins \
        ${FEATURES} \
        --root /builder/release
RUN --mount=type=cache,target=/cargo CARGO_HOME=/cargo \
        RUSTFLAGS="-C target-cpu=native" \
        cargo install \
        --path=applications \
        --bins \
        ${FEATURES} \
        --profile profiling \
        --root /builder/profiling

# Use Ubuntu as the runtime image
FROM ubuntu:24.04

WORKDIR /honeycomb
ARG USE_CUDA=false
ENV USE_CUDA=${USE_CUDA}

# Install performance tools & dependencies
RUN apt-get update && apt-get install -y \
    linux-tools-generic \
    heaptrack \
    libhwloc-dev libudev-dev

RUN if [ "$INSTALL_CUDA" = "true" ]; then \
        apt-get install -y nvidia-cuda-toolkit; \
    else \ 
        echo "Skipping CUDA installation";

# Copy useful stuff
COPY --from=builder /builder/release/bin   /honeycomb/rbin
COPY --from=builder /builder/profiling/bin /honeycomb/pbin
