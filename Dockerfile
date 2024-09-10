FROM rust:1.81-slim-bullseye

# RUN apt-get update -y && \
#     apt-get upgrade -y
    # apt-get install -y default-mysql-client
    # apt-get install -y mariadb-client

ENV CARGO_TARGET_DIR=/tmp/target

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "1001001" \
    appuser
    
RUN chown -R appuser:root *
USER appuser

RUN apt-get install libxcb-xfixes0-dev
RUN apt-get install libxcb-shape0-dev
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch

RUN USER=root cargo new --bin app

WORKDIR /app

COPY ./Cargo.toml Cargo.toml
COPY . .
# COPY ./Cargo.lock Cargo.lock

