# This dockerfile is meant to be built from the context of the parent folder
# e.g.: docker build -t duty-duck-server:latest -f server/Dockerfile .

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner

# Copy all the Rust source code (omit the front-end stuff)
COPY ./protos ./protos
COPY ./components/server ./components/server
COPY ./components/cli ./components/cli
COPY ./components/api-client-rs ./components/api-client-rs

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:latest AS build-env

RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /app
COPY ./Cargo.toml ./Cargo.lock ./

COPY ./protos ./protos
COPY ./components/server ./components/server
COPY ./components/cli ./components/cli
COPY ./components/api-client-rs ./components/api-client-rs

RUN cargo build --release --bin server

FROM gcr.io/distroless/cc-debian12
COPY --from=build-env /app/target/release/server /app/server
CMD ["/app/server"]