FROM rust:latest

RUN apt-get update -y && \
    apt-get upgrade -y && \
    apt-get install -y default-mysql-client
#    apt-get install -y mariadb-client

ENV CARGO_TARGET_DIR=/tmp/target

RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch

RUN USER=root cargo new --bin app

RUN curl -o /wait-for-it.sh https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh
RUN chmod +x /wait-for-it.sh

WORKDIR /app

COPY ./Cargo.toml Cargo.toml
COPY . .
# COPY ./Cargo.lock Cargo.lock

