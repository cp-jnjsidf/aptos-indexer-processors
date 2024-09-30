### Indexer Processor Image ###

# Stage 1: Build the binary

FROM rust:slim-bullseye as builder

WORKDIR /app

# Install necessary packages
RUN apt-get update && apt-get install -y \
    cmake curl clang git pkg-config libssl-dev libdw-dev libpq-dev lld

# Copy only the Cargo files first to leverage Docker's cache
COPY Cargo.toml Cargo.lock ./
COPY integration-tests/Cargo.toml integration-tests/
COPY processor/Cargo.toml processor/
COPY testing-transactions/Cargo.toml testing-transactions/
COPY server-framework/Cargo.toml server-framework/
COPY indexer-metrics/Cargo.toml indexer-metrics/
COPY sdk-processor/Cargo.toml sdk-processor/
COPY moving-average/Cargo.toml moving-average/

# Create hardcoded dummy files for known binaries and libraries
RUN mkdir -p integration-tests/src && echo "pub fn dummy_lib() {}" > integration-tests/src/lib.rs && \
    mkdir -p processor/src && echo "fn main() { println!(\"Processor main\"); }" > processor/src/main.rs && \
    mkdir -p testing-transactions/src && echo "pub fn dummy_lib() {}" > testing-transactions/src/lib.rs && \
    mkdir -p server-framework/src && echo "pub fn dummy_lib() {}" > server-framework/src/lib.rs && \
    mkdir -p indexer-metrics/src && echo "fn main() { println!(\"Indexer Metrics main\"); }" > indexer-metrics/src/main.rs && \
    mkdir -p sdk-processor/src && echo "fn main() { println!(\"SDK Processor main\"); }" > sdk-processor/src/main.rs && \
    mkdir -p moving-average/src && echo "pub fn dummy_lib() {}" > moving-average/src/lib.rs

# Build the dependencies first to leverage Docker caching
RUN cargo build --release --locked

# Now copy the rest of the source files
COPY --link . .

# Build the individual components
RUN cargo build --locked --release -p processor
RUN cp target/release/processor /usr/local/bin
RUN cargo build --locked --release -p indexer-metrics
RUN cp target/release/indexer-metrics /usr/local/bin
RUN cargo build --locked --release -p sdk-processor
RUN cp target/release/sdk-processor /usr/local/bin

# Stage 2: Create the final image
FROM debian:bullseye-slim

COPY --from=builder /usr/local/bin/processor /usr/local/bin
COPY --from=builder /usr/local/bin/indexer-metrics /usr/local/bin
COPY --from=builder /usr/local/bin/sdk-processor /usr/local/bin

# Install the runtime dependencies
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install --no-install-recommends -y \
        libssl1.1 \
        ca-certificates \
        net-tools \
        tcpdump \
        iproute2 \
        netcat \
        libdw-dev \
        libpq-dev \
        curl

# Set environment variables
ENV RUST_LOG_FORMAT=json

# The health check ports
EXPOSE 8087
EXPOSE 8088

ENTRYPOINT ["/usr/local/bin/processor"]