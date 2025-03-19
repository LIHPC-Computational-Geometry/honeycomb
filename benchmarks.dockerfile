FROM rust:1.85 AS builder

WORKDIR /builder

# Copy files from the repo
COPY . .

# Install dependencuies
# RUN apt-get update

# Build binaries
RUN --mount=type=cache,target=/cargo CARGO_HOME=/cargo \
        cargo install \
        --path=benches \
        --bins \
        --root /builder/release
RUN --mount=type=cache,target=/cargo CARGO_HOME=/cargo \
        cargo install \
        --path=benches \
        --bins \
        --profile profiling \
        --root /builder/profiling

# Use Ubuntu as the runtime image
FROM ubuntu:22.04

# Install performance tools
RUN apt-get update && apt-get install -y \
    linux-tools-generic \
    heaptrack

WORKDIR /honeycomb

# Fetch input meshes
# for some reason this doesn't resolve
# ADD https://github.com/imrn99/meshing-samples.git /honeycomb/meshes/

# Copy useful stuff
COPY --from=builder /builder/release/bin /honeycomb/rbin
COPY --from=builder /builder/profiling/bin /honeycomb/pbin
