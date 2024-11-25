# This dockerfile is meant to be built from the context of the parent folder
# e.g.: docker build -t duty-duck-server:latest -f server/Dockerfile .
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