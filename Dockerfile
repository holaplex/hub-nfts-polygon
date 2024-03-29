FROM rust:1.69.0-bullseye as chef
RUN cargo install cargo-chef --locked

WORKDIR /app

RUN apt-get update -y && \
  apt-get install -y --no-install-recommends \
    cmake \
    g++ \
    libsasl2-dev \
    libssl-dev \
    libudev-dev \
    wget \
    pkg-config \
  && \
  rm -rf /var/lib/apt/lists/*

COPY ci/get-protoc.sh ./
RUN chmod +x get-protoc.sh
RUN /app/get-protoc.sh

FROM chef AS planner

COPY Cargo.* ./
COPY consumer consumer
COPY core core
COPY entity entity
COPY migration migration
COPY evm-contracts-build evm-contracts-build
COPY indexer indexer

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY Cargo.* ./

COPY consumer consumer
COPY core core
COPY entity entity
COPY migration migration
COPY evm-contracts-build evm-contracts-build
COPY indexer indexer


FROM builder AS builder-hub-nfts-polygon
RUN cargo build --release --bin holaplex-hub-nfts-polygon

FROM builder AS builder-hub-nfts-polygon-indexer
RUN cargo build --release --bin holaplex-hub-nfts-polygon-indexer

FROM builder AS builder-migration
RUN cargo build --release --bin migration


FROM debian:bullseye-slim as base

WORKDIR /app

ENV TZ=Etc/UTC
ENV APP_USER=runner

RUN apt-get update -y && \
  apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl1.1 \
  && rm -rf /var/lib/apt/lists/*

RUN groupadd $APP_USER \
    && useradd --uid 10000 -g $APP_USER $APP_USER \
    && mkdir -p bin

RUN chown -R $APP_USER:$APP_USER bin

USER 10000

FROM base AS hub-nfts-polygon

COPY --from=builder-hub-nfts-polygon /app/target/release/holaplex-hub-nfts-polygon /usr/local/bin
CMD ["/usr/local/bin/holaplex-hub-nfts-polygon"]

FROM base AS migrator

COPY --from=builder-migration /app/target/release/migration bin/
CMD ["bin/migration"]

FROM base AS polygon-indexer

COPY --from=builder-hub-nfts-polygon-indexer /app/target/release/holaplex-hub-nfts-polygon-indexer /usr/local/bin
CMD ["/usr/local/bin/holaplex-hub-nfts-polygon-indexer"]
