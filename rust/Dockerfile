### Indexer Processor Image ###

# Stage 1: Install dependencies and tools
FROM rust:slim-bullseye as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y cmake curl clang git pkg-config libssl-dev libdw-dev libpq-dev lld

# Install grpcurl
RUN curl -sSL "https://github.com/fullstorydev/grpcurl/releases/download/v1.8.7/grpcurl_1.8.7_linux_x86_64.tar.gz" | tar -xz -C /usr/local/bin

# Copy root Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy necessary Cargo files for each subproject
COPY processor/Cargo.toml processor/Cargo.lock ./processor/
COPY sdk-processor/Cargo.toml ./sdk-processor/
COPY indexer-metrics/Cargo.toml ./indexer-metrics/
COPY server-framework/Cargo.toml ./server-framework/
COPY integration-tests/Cargo.toml ./integration-tests/
COPY moving-average/Cargo.toml ./moving-average/

# Copy only the lib.rs files for each project
COPY processor/src/lib.rs ./processor/src/lib.rs
COPY sdk-processor/src/lib.rs ./sdk-processor/src/lib.rs
COPY indexer-metrics/src/lib.rs ./indexer-metrics/src/lib.rs
COPY server-framework/src/lib.rs ./server-framework/src/lib.rs
COPY integration-tests/src/lib.rs ./integration-tests/src/lib.rs
COPY moving-average/src/lib.rs ./moving-average/src/lib.rs


ENV CARGO_NET_GIT_FETCH_WITH_CLI true
RUN cargo build --locked --release -p processor
RUN cp target/release/processor /usr/local/bin
RUN cargo build --locked --release -p indexer-metrics
RUN cp target/release/indexer-metrics /usr/local/bin
RUN cargo build --locked --release -p sdk-processor
RUN cp target/release/sdk-processor /usr/local/bin

# Copy the entire source code after building
COPY --link . /app

# add build info
ARG GIT_TAG
ENV GIT_TAG ${GIT_TAG}
ARG GIT_BRANCH
ENV GIT_BRANCH ${GIT_BRANCH}
ARG GIT_SHA
ENV GIT_SHA ${GIT_SHA}

# Stage 2: Create the final image

FROM debian:bullseye-slim

COPY --from=builder /usr/local/bin/processor /usr/local/bin
COPY --from=builder /usr/local/bin/indexer-metrics /usr/local/bin
COPY --from=builder /usr/local/bin/sdk-processor /usr/local/bin

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

ENV RUST_LOG_FORMAT=json

# add build info
ARG GIT_TAG
ENV GIT_TAG ${GIT_TAG}
ARG GIT_BRANCH
ENV GIT_BRANCH ${GIT_BRANCH}
ARG GIT_SHA
ENV GIT_SHA ${GIT_SHA}

# The health check port
EXPOSE 8087

ENTRYPOINT ["/usr/local/bin/processor"]
