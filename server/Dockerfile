# This dockerfile is meant to be built from the context of the parent folder
# e.g.: docker build -t duty-duck-server:latest -f server/Dockerfile .
FROM rust:latest AS builder

RUN update-ca-certificates
RUN mkdir -p /usr/src/dutyduck/server
WORKDIR /usr/src/dutyduck/server
COPY ./server .
RUN cargo build --release

FROM gcr.io/distroless/cc


WORKDIR /usr/src/dutyduck/server

# Copy our build
COPY --from=builder /usr/src/dutyduck/server/target/release/server ./

CMD ["server"]