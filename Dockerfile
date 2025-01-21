FROM rust:1.84 as builder

WORKDIR /builder

# Install git for cloning
RUN apt-get update && apt-get install -y git

# Clone repositories (honeycomb + inputs)
RUN git clone https://github.com/LIHPC-Computational-Geometry/honeycomb .
RUN git clone https://github.com/imrn99/meshing-samples

# Build the project
RUN cargo build --benches --release -p honeycomb-benches
RUN cargo build --bins --release
RUN cargo build --bins --profile profiling

# Use Ubuntu as the runtime image
FROM ubuntu:22.04

# Install performance tools
RUN apt-get update && apt-get install -y \
    linux-tools-generic \
    heaptrack

WORKDIR /honeycomb

# Copy useful stuff
COPY --from=builder /builder/meshing-samples /honeycomb/
COPY --from=builder /builder/target/release /honeycomb/
COPY --from=builder /builder/target/profiling /honeycomb/

