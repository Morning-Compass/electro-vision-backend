FROM rust:1.82-slim-bullseye

RUN apt-get update -y && \
    apt-get upgrade -y
RUN apt-get install libpq-dev -y
# apt-get install -y default-mysql-client
# apt-get install -y mariadb-client

ENV CARGO_TARGET_DIR=/tmp/target
WORKDIR /app
COPY ./Cargo.toml Cargo.toml
COPY . .

RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
