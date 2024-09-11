FROM rust:1.81-slim-bullseye

# RUN apt-get update -y && \
#     apt-get upgrade -y
    # apt-get install -y default-mysql-client
    # apt-get install -y mariadb-client

ENV CARGO_TARGET_DIR=/tmp/target

# RUN apt-get install libxcb-xfixes0-dev
# RUN apt-get install libxcb-shape0-dev
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
RUN pg_ctl -D /var/lib/postgresql/data -l logfile start
RUN diesel migration run && cargo run
RUN USER=root cargo new --bin app

WORKDIR /app
RUN echo "works!!!!!!!!"
COPY ./Cargo.toml Cargo.toml
COPY . .
# COPY ./Cargo.lock Cargo.lock

