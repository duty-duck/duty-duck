FROM rust:1-bullseye
WORKDIR /home/dutyduck-migrations
RUN cargo install sqlx-cli@^0.7
COPY server/migrations ./migrations
ENTRYPOINT [ "sqlx", "migrate", "run" ]