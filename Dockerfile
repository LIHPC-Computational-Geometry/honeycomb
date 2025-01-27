FROM rust:1.84 as builder

WORKDIR /builder

# Install dependencuies
# RUN apt-get update

# Build the project
RUN --mount=type=cache,target=/cargo CARGO_HOME=/cargo cargo build --benches --release -p honeycomb-benches
RUN --mount=type=cache,target=/cargo CARGO_HOME=/cargo cargo build --bins --release
RUN --mount=type=cache,target=/cargo CARGO_HOME=/cargo cargo build --bins --profile profiling

# Use Ubuntu as the runtime image
FROM ubuntu:22.04

# Install performance tools
RUN apt-get update && apt-get install -y \
    linux-tools-generic \
    heaptrack

WORKDIR /honeycomb

# Fetch input meshes
ADD git@github.com:imrn99/meshing-samples.git /honeycomb/meshes/

# Copy useful stuff
COPY --from=builder /builder/target/release /honeycomb/
COPY --from=builder /builder/target/profiling /honeycomb/

