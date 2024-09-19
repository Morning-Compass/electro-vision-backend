FROM rust:1.81-slim-bullseye

RUN apt-get update -y && \
    apt-get upgrade -y
# apt-get install -y default-mysql-client
# apt-get install -y mariadb-client

ENV CARGO_TARGET_DIR=/tmp/target
WORKDIR /app
COPY ./Cargo.toml Cargo.toml
COPY . .

RUN apt-get install libpq-dev -y
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
RUN USER=root cargo new --bin app
