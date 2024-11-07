# This dockerfile is meant to be built from the context of the parent folder
# e.g.: docker build -t duty-duck-server:latest -f server/Dockerfile .
FROM rust:latest AS build-env

RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /app/server
COPY ./protos ../protos

RUN cargo init --lib --vcs none

COPY ./server/Cargo.toml ./server/Cargo.lock ./server/build.rs ./
RUN cargo build --release --lib

COPY ./server .
RUN touch src/main.rs && cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build-env /app/target/release/server /app/server
CMD ["/app/server"]